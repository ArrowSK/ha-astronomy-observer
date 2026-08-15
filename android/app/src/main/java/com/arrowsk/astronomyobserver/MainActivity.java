package com.arrowsk.astronomyobserver;

import android.Manifest;
import android.app.Activity;
import android.app.AlertDialog;
import android.content.Intent;
import android.content.pm.PackageManager;
import android.location.Location;
import android.location.LocationListener;
import android.location.LocationManager;
import android.net.Uri;
import android.os.Build;
import android.os.Bundle;
import android.os.Handler;
import android.os.Looper;
import android.provider.Settings;
import android.view.ViewGroup;
import android.view.WindowInsets;
import android.webkit.WebChromeClient;
import android.webkit.WebResourceRequest;
import android.webkit.WebResourceResponse;
import android.webkit.WebSettings;
import android.webkit.WebView;
import android.webkit.WebViewClient;
import android.widget.ScrollView;
import android.widget.TextView;
import android.widget.Toast;

import org.json.JSONObject;

import java.io.File;
import java.io.FileOutputStream;
import java.io.IOException;
import java.io.InputStream;
import java.nio.charset.StandardCharsets;
import java.time.ZoneId;

public final class MainActivity extends Activity {
    private static final int LOCATION_REQUEST = 1001;
    private static final String APP_URL = "file:///android_asset/index.html";
    private static final String[] RUNTIME_ASSETS = {
            "catalog.tsv",
            "meteor_showers.csv",
            "world_atlas_3min.bin",
            "world_atlas_3min.json",
            "WORLD_ATLAS_NOTICE.md"
    };

    private WebView webView;
    private AndroidBridge bridge;
    private LocationManager locationManager;
    private LocationListener pendingLocationListener;
    private final Handler handler = new Handler(Looper.getMainLooper());

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);

        try {
            File resourceDir = new File(getFilesDir(), "resources");
            File dataDir = new File(getFilesDir(), "data");
            File configDir = new File(getFilesDir(), "config");
            ensureDirectory(resourceDir);
            ensureDirectory(dataDir);
            ensureDirectory(configDir);
            copyRuntimeAssets(resourceDir);
            NativeBridge.initialize(resourceDir.getAbsolutePath(), dataDir.getAbsolutePath(), configDir.getAbsolutePath());
        } catch (Throwable error) {
            new AlertDialog.Builder(this)
                    .setTitle("Astronomy Observer could not start")
                    .setMessage(error.getMessage() == null ? error.toString() : error.getMessage())
                    .setPositiveButton("Close", (dialog, which) -> finish())
                    .setCancelable(false)
                    .show();
            return;
        }

        locationManager = (LocationManager) getSystemService(LOCATION_SERVICE);
        webView = new WebView(this);
        webView.setLayoutParams(new ViewGroup.LayoutParams(
                ViewGroup.LayoutParams.MATCH_PARENT,
                ViewGroup.LayoutParams.MATCH_PARENT));
        configureWebView(webView);
        setContentView(webView);
        applySystemBarPadding(webView);

        bridge = new AndroidBridge(this, webView);
        webView.addJavascriptInterface(bridge, "AstronomyAndroid");
        webView.loadUrl(APP_URL);
    }

    private void ensureDirectory(File directory) throws IOException {
        if (!directory.isDirectory() && !directory.mkdirs()) {
            throw new IOException("Could not create " + directory.getAbsolutePath());
        }
    }

    private void copyRuntimeAssets(File resourceDir) throws IOException {
        for (String name : RUNTIME_ASSETS) {
            File target = new File(resourceDir, name);
            if (target.isFile() && target.length() > 0) {
                continue;
            }
            File temporary = new File(resourceDir, name + ".tmp");
            try (InputStream input = getAssets().open(name);
                 FileOutputStream output = new FileOutputStream(temporary)) {
                byte[] buffer = new byte[64 * 1024];
                int read;
                while ((read = input.read(buffer)) >= 0) {
                    output.write(buffer, 0, read);
                }
            }
            if (target.exists() && !target.delete()) {
                throw new IOException("Could not replace " + target.getName());
            }
            if (!temporary.renameTo(target)) {
                throw new IOException("Could not install " + target.getName());
            }
        }
    }

    private void configureWebView(WebView view) {
        WebSettings settings = view.getSettings();
        settings.setJavaScriptEnabled(true);
        settings.setDomStorageEnabled(true);
        settings.setGeolocationEnabled(false);
        settings.setAllowContentAccess(false);
        settings.setAllowFileAccess(true);
        settings.setAllowFileAccessFromFileURLs(false);
        settings.setAllowUniversalAccessFromFileURLs(false);
        settings.setMixedContentMode(WebSettings.MIXED_CONTENT_NEVER_ALLOW);
        settings.setSaveFormData(false);
        settings.setBlockNetworkLoads(true);

        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            WebView.startSafeBrowsing(this, null);
        }

        view.setWebChromeClient(new WebChromeClient());
        view.setWebViewClient(new WebViewClient() {
            @Override
            public boolean shouldOverrideUrlLoading(WebView ignored, WebResourceRequest request) {
                Uri uri = request.getUrl();
                String value = uri.toString();
                if (value.startsWith("file:///android_asset/")) {
                    return false;
                }
                if ("https".equalsIgnoreCase(uri.getScheme()) || "http".equalsIgnoreCase(uri.getScheme())) {
                    try {
                        startActivity(new Intent(Intent.ACTION_VIEW, uri));
                    } catch (Exception error) {
                        Toast.makeText(MainActivity.this, "No browser is available for this link.", Toast.LENGTH_SHORT).show();
                    }
                }
                return true;
            }

            @Override
            public WebResourceResponse shouldInterceptRequest(WebView ignored, WebResourceRequest request) {
                Uri uri = request.getUrl();
                String scheme = uri.getScheme();
                if ("http".equalsIgnoreCase(scheme) || "https".equalsIgnoreCase(scheme)) {
                    return new WebResourceResponse(
                            "text/plain",
                            "UTF-8",
                            403,
                            "Blocked",
                            java.util.Collections.emptyMap(),
                            new java.io.ByteArrayInputStream(new byte[0]));
                }
                return super.shouldInterceptRequest(ignored, request);
            }
        });
    }

    private void applySystemBarPadding(WebView view) {
        view.setOnApplyWindowInsetsListener((v, insets) -> {
            int top;
            int bottom;
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.R) {
                android.graphics.Insets bars = insets.getInsets(WindowInsets.Type.systemBars());
                top = bars.top;
                bottom = bars.bottom;
            } else {
                top = insets.getSystemWindowInsetTop();
                bottom = insets.getSystemWindowInsetBottom();
            }
            v.setPadding(0, top, 0, bottom);
            return insets;
        });
    }

    void requestCurrentLocation() {
        if (checkSelfPermission(Manifest.permission.ACCESS_FINE_LOCATION) != PackageManager.PERMISSION_GRANTED
                && checkSelfPermission(Manifest.permission.ACCESS_COARSE_LOCATION) != PackageManager.PERMISSION_GRANTED) {
            requestPermissions(new String[]{
                    Manifest.permission.ACCESS_FINE_LOCATION,
                    Manifest.permission.ACCESS_COARSE_LOCATION
            }, LOCATION_REQUEST);
            return;
        }
        locateNow();
    }

    @Override
    public void onRequestPermissionsResult(int requestCode, String[] permissions, int[] grantResults) {
        super.onRequestPermissionsResult(requestCode, permissions, grantResults);
        if (requestCode != LOCATION_REQUEST) {
            return;
        }
        boolean granted = false;
        for (int result : grantResults) {
            granted |= result == PackageManager.PERMISSION_GRANTED;
        }
        if (granted) {
            locateNow();
        } else {
            sendLocationError("Location permission was not granted. You can enter the observing site manually.");
        }
    }

    private void locateNow() {
        if (checkSelfPermission(Manifest.permission.ACCESS_FINE_LOCATION) != PackageManager.PERMISSION_GRANTED
                && checkSelfPermission(Manifest.permission.ACCESS_COARSE_LOCATION) != PackageManager.PERMISSION_GRANTED) {
            sendLocationError("Location permission is unavailable.");
            return;
        }

        String provider = locationManager.isProviderEnabled(LocationManager.GPS_PROVIDER)
                ? LocationManager.GPS_PROVIDER
                : LocationManager.NETWORK_PROVIDER;
        Location last = locationManager.getLastKnownLocation(provider);
        if (last != null && System.currentTimeMillis() - last.getTime() < 30 * 60 * 1000L) {
            sendLocation(last);
            return;
        }

        if (!locationManager.isProviderEnabled(provider)) {
            sendLocationError("Location is switched off. Enter the site manually or enable device location.");
            return;
        }

        if (pendingLocationListener != null) {
            locationManager.removeUpdates(pendingLocationListener);
        }
        pendingLocationListener = location -> {
            handler.removeCallbacksAndMessages(null);
            if (pendingLocationListener != null) {
                locationManager.removeUpdates(pendingLocationListener);
                pendingLocationListener = null;
            }
            sendLocation(location);
        };
        locationManager.requestSingleUpdate(provider, pendingLocationListener, Looper.getMainLooper());
        handler.postDelayed(() -> {
            if (pendingLocationListener != null) {
                locationManager.removeUpdates(pendingLocationListener);
                pendingLocationListener = null;
                sendLocationError("The phone did not return a location in time. Try again or enter the site manually.");
            }
        }, 12000L);
    }

    private void sendLocation(Location location) {
        try {
            JSONObject value = new JSONObject();
            value.put("ok", true);
            value.put("latitude", location.getLatitude());
            value.put("longitude", location.getLongitude());
            value.put("elevation_m", location.hasAltitude() ? location.getAltitude() : 0.0);
            value.put("label", "Current location");
            value.put("timezone", ZoneId.systemDefault().getId());
            evaluateLocationCallback(value);
        } catch (Exception error) {
            sendLocationError(error.getMessage() == null ? error.toString() : error.getMessage());
        }
    }

    private void sendLocationError(String message) {
        try {
            JSONObject value = new JSONObject();
            value.put("ok", false);
            value.put("error", message);
            evaluateLocationCallback(value);
        } catch (Exception ignored) {
            Toast.makeText(this, message, Toast.LENGTH_LONG).show();
        }
    }

    private void evaluateLocationCallback(JSONObject value) {
        if (webView != null) {
            webView.evaluateJavascript("window.androidLocationResult(" + value + ");", null);
        }
    }

    void showLicences() {
        String[] assets = {
                "legal-summary.txt",
                "project-license.txt",
                "astronomy-engine-license.txt",
                "WORLD_ATLAS_NOTICE.md"
        };
        StringBuilder text = new StringBuilder();
        for (String asset : assets) {
            try {
                if (text.length() > 0) {
                    text.append("\n\n────────────────────────\n\n");
                }
                text.append(readAssetText(asset));
            } catch (IOException error) {
                text.append("\n\nCould not read ").append(asset).append(": ").append(error.getMessage());
            }
        }
        TextView body = new TextView(this);
        int padding = (int) (18 * getResources().getDisplayMetrics().density);
        body.setPadding(padding, padding, padding, padding);
        body.setText(text.toString());
        body.setTextIsSelectable(true);
        body.setTextSize(13f);
        ScrollView scroll = new ScrollView(this);
        scroll.addView(body);
        new AlertDialog.Builder(this)
                .setTitle("About & licences")
                .setView(scroll)
                .setPositiveButton("Close", null)
                .show();
    }

    private String readAssetText(String name) throws IOException {
        try (InputStream input = getAssets().open(name)) {
            byte[] bytes = new byte[8192];
            StringBuilder output = new StringBuilder();
            int read;
            while ((read = input.read(bytes)) >= 0) {
                output.append(new String(bytes, 0, read, StandardCharsets.UTF_8));
            }
            return output.toString();
        }
    }

    @Override
    public void onBackPressed() {
        if (webView != null && webView.canGoBack()) {
            webView.goBack();
        } else {
            super.onBackPressed();
        }
    }

    @Override
    protected void onDestroy() {
        handler.removeCallbacksAndMessages(null);
        if (pendingLocationListener != null && locationManager != null) {
            try {
                locationManager.removeUpdates(pendingLocationListener);
            } catch (SecurityException ignored) {
                // Activity is already closing.
            }
            pendingLocationListener = null;
        }
        if (bridge != null) {
            bridge.shutdown();
        }
        if (webView != null) {
            webView.removeJavascriptInterface("AstronomyAndroid");
            webView.destroy();
        }
        super.onDestroy();
    }
}
