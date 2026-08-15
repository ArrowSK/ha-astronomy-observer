#include <jni.h>
#include <math.h>
#include <stdarg.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "astronomy.h"

extern char *ao_initialize(const char *resource_dir, const char *data_dir, const char *config_dir);
extern char *ao_calculate_json(const char *input);
extern void ao_free_string(char *value);

static const astro_body_t bodies[] = {
    BODY_SUN, BODY_MOON, BODY_MERCURY, BODY_VENUS, BODY_MARS,
    BODY_JUPITER, BODY_SATURN, BODY_URANUS, BODY_NEPTUNE, BODY_PLUTO
};

typedef struct {
    char *data;
    size_t length;
    size_t capacity;
} text_buffer_t;

static int appendf(text_buffer_t *buffer, const char *format, ...) {
    va_list args;
    va_start(args, format);
    va_list copy;
    va_copy(copy, args);
    int needed = vsnprintf(NULL, 0, format, copy);
    va_end(copy);
    if (needed < 0) {
        va_end(args);
        return 1;
    }
    size_t required = buffer->length + (size_t)needed + 1;
    if (required > buffer->capacity) {
        size_t capacity = buffer->capacity ? buffer->capacity : 4096;
        while (capacity < required) capacity *= 2;
        char *replacement = (char *)realloc(buffer->data, capacity);
        if (!replacement) {
            va_end(args);
            return 1;
        }
        buffer->data = replacement;
        buffer->capacity = capacity;
    }
    vsnprintf(buffer->data + buffer->length, buffer->capacity - buffer->length, format, args);
    va_end(args);
    buffer->length += (size_t)needed;
    return 0;
}

static int emit_row(text_buffer_t *buffer, int64_t epoch_seconds, astro_observer_t observer) {
    astro_time_t linux_epoch = Astronomy_MakeTime(1970, 1, 1, 0, 0, 0.0);
    astro_time_t time = Astronomy_AddDays(linux_epoch, (double)epoch_seconds / 86400.0);
    astro_rotation_t ecl_rot = Astronomy_Rotation_EQJ_ECL();
    astro_vector_t earth_eqj = Astronomy_HelioVector(BODY_EARTH, time);
    astro_vector_t earth_ecl = Astronomy_RotateVector(ecl_rot, earth_eqj);
    if (earth_ecl.status != ASTRO_SUCCESS) return 20 + earth_ecl.status;

    if (appendf(buffer, "E,%lld,%.10f,%.10f,%.10f\n", (long long)epoch_seconds,
                earth_ecl.x, earth_ecl.y, earth_ecl.z)) return 10;

    size_t count = sizeof(bodies) / sizeof(bodies[0]);
    for (size_t i = 0; i < count; ++i) {
        astro_body_t body = bodies[i];
        astro_equatorial_t eq = Astronomy_Equator(body, &time, observer, EQUATOR_OF_DATE, ABERRATION);
        if (eq.status != ASTRO_SUCCESS) return 40 + eq.status;
        astro_horizon_t hor = Astronomy_Horizon(&time, observer, eq.ra, eq.dec, REFRACTION_NORMAL);
        double mag = NAN;
        double fraction = NAN;
        if (body == BODY_SUN) {
            mag = -26.74;
            fraction = 1.0;
        } else {
            astro_illum_t illum = Astronomy_Illumination(body, time);
            if (illum.status == ASTRO_SUCCESS) {
                mag = illum.mag;
                fraction = (1.0 + cos(illum.phase_angle * (3.14159265358979323846 / 180.0))) / 2.0;
            }
        }
        if (appendf(buffer, "B,%lld,%s,%.9f,%.9f,%.6f,%.6f,%.4f,%.6f\n",
                    (long long)epoch_seconds, Astronomy_BodyName(body), eq.ra, eq.dec,
                    hor.azimuth, hor.altitude, mag, fraction)) return 10;
    }
    return 0;
}

char *ao_astro_calculate(double latitude, double longitude, double elevation,
                         const int64_t *epochs, size_t count) {
    if (!epochs || count == 0 || latitude < -90.0 || latitude > 90.0 ||
        longitude < -180.0 || longitude > 180.0) {
        return NULL;
    }
    text_buffer_t buffer = {0};
    astro_observer_t observer = Astronomy_MakeObserver(latitude, longitude, elevation);
    for (size_t i = 0; i < count; ++i) {
        if (emit_row(&buffer, epochs[i], observer) != 0) {
            free(buffer.data);
            return NULL;
        }
    }
    if (!buffer.data) {
        buffer.data = (char *)calloc(1, 1);
    }
    return buffer.data;
}

void ao_astro_free(char *value) {
    free(value);
}

static jstring java_string(JNIEnv *env, char *value) {
    if (!value) return (*env)->NewStringUTF(env, "native bridge returned no result");
    jstring result = (*env)->NewStringUTF(env, value);
    ao_free_string(value);
    return result;
}

JNIEXPORT jstring JNICALL
Java_com_arrowsk_astronomyobserver_NativeBridge_nativeInitialize(
    JNIEnv *env, jclass clazz, jstring resource_dir, jstring data_dir, jstring config_dir) {
    (void)clazz;
    if (!resource_dir || !data_dir || !config_dir) {
        return (*env)->NewStringUTF(env, "native initialization received a null path");
    }
    const char *resource = (*env)->GetStringUTFChars(env, resource_dir, NULL);
    const char *data = (*env)->GetStringUTFChars(env, data_dir, NULL);
    const char *config = (*env)->GetStringUTFChars(env, config_dir, NULL);
    if (!resource || !data || !config) {
        if (resource) (*env)->ReleaseStringUTFChars(env, resource_dir, resource);
        if (data) (*env)->ReleaseStringUTFChars(env, data_dir, data);
        if (config) (*env)->ReleaseStringUTFChars(env, config_dir, config);
        return (*env)->NewStringUTF(env, "native initialization could not read a path");
    }
    char *value = ao_initialize(resource, data, config);
    (*env)->ReleaseStringUTFChars(env, resource_dir, resource);
    (*env)->ReleaseStringUTFChars(env, data_dir, data);
    (*env)->ReleaseStringUTFChars(env, config_dir, config);
    return java_string(env, value);
}

JNIEXPORT jstring JNICALL
Java_com_arrowsk_astronomyobserver_NativeBridge_nativeCalculate(
    JNIEnv *env, jclass clazz, jstring request_json) {
    (void)clazz;
    if (!request_json) {
        return (*env)->NewStringUTF(env, "{\"ok\":false,\"error\":\"empty request\"}");
    }
    const char *request = (*env)->GetStringUTFChars(env, request_json, NULL);
    if (!request) {
        return (*env)->NewStringUTF(env, "{\"ok\":false,\"error\":\"request is not readable\"}");
    }
    char *value = ao_calculate_json(request);
    (*env)->ReleaseStringUTFChars(env, request_json, request);
    return java_string(env, value);
}
