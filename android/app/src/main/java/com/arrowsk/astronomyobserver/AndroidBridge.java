package com.arrowsk.astronomyobserver;

import android.webkit.JavascriptInterface;
import android.webkit.WebView;

import org.json.JSONObject;

import java.util.concurrent.ExecutorService;
import java.util.concurrent.Executors;

final class AndroidBridge {
    private final MainActivity activity;
    private final WebView webView;
    private final ExecutorService executor = Executors.newSingleThreadExecutor();

    AndroidBridge(MainActivity activity, WebView webView) {
        this.activity = activity;
        this.webView = webView;
    }

    @JavascriptInterface
    public void calculate(String requestJson) {
        executor.execute(() -> {
            String result;
            try {
                result = NativeBridge.calculate(requestJson);
            } catch (Throwable error) {
                JSONObject fallback = new JSONObject();
                try {
                    fallback.put("ok", false);
                    fallback.put("error", error.getMessage() == null ? error.toString() : error.getMessage());
                } catch (Exception ignored) {
                    // JSONObject with two primitive fields should not fail.
                }
                result = fallback.toString();
            }
            final String payload = result;
            activity.runOnUiThread(() -> webView.evaluateJavascript(
                    "window.androidCalculationResult(" + JSONObject.quote(payload) + ");", null));
        });
    }

    @JavascriptInterface
    public void requestLocation() {
        activity.runOnUiThread(activity::requestCurrentLocation);
    }

    @JavascriptInterface
    public void showLicences() {
        activity.runOnUiThread(activity::showLicences);
    }

    @JavascriptInterface
    public String version() {
        return BuildConfig.VERSION_NAME;
    }

    void shutdown() {
        executor.shutdownNow();
    }
}
