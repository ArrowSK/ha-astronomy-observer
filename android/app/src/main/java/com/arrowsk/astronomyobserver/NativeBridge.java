package com.arrowsk.astronomyobserver;

final class NativeBridge {
    static {
        System.loadLibrary("astronomy_observer_android");
    }

    private NativeBridge() {}

    static void initialize(String resourceDir, String dataDir, String configDir) {
        String error = nativeInitialize(resourceDir, dataDir, configDir);
        if (error != null && !error.isEmpty()) {
            throw new IllegalStateException(error);
        }
    }

    static String calculate(String requestJson) {
        return nativeCalculate(requestJson);
    }

    private static native String nativeInitialize(String resourceDir, String dataDir, String configDir);
    private static native String nativeCalculate(String requestJson);
}
