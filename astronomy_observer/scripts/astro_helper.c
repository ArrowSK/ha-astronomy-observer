#include <errno.h>
#include <math.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "astronomy.h"

static const astro_body_t bodies[] = {
    BODY_SUN, BODY_MOON, BODY_MERCURY, BODY_VENUS, BODY_MARS,
    BODY_JUPITER, BODY_SATURN, BODY_URANUS, BODY_NEPTUNE, BODY_PLUTO
};

static int emit_row(int64_t epoch_seconds, astro_observer_t observer) {
    astro_time_t linux_epoch = Astronomy_MakeTime(1970, 1, 1, 0, 0, 0.0);
    astro_time_t time = Astronomy_AddDays(linux_epoch, (double)epoch_seconds / 86400.0);
    astro_rotation_t ecl_rot = Astronomy_Rotation_EQJ_ECL();
    astro_vector_t earth_eqj = Astronomy_HelioVector(BODY_EARTH, time);
    astro_vector_t earth_ecl = Astronomy_RotateVector(ecl_rot, earth_eqj);
    if (earth_ecl.status != ASTRO_SUCCESS) return 20 + earth_ecl.status;

    printf("E,%lld,%.10f,%.10f,%.10f\n", (long long)epoch_seconds, earth_ecl.x, earth_ecl.y, earth_ecl.z);

    size_t n = sizeof(bodies) / sizeof(bodies[0]);
    for (size_t i = 0; i < n; ++i) {
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
        printf("B,%lld,%s,%.9f,%.9f,%.6f,%.6f,%.4f,%.6f\n",
               (long long)epoch_seconds, Astronomy_BodyName(body), eq.ra, eq.dec,
               hor.azimuth, hor.altitude, mag, fraction);
    }
    return 0;
}

int main(int argc, char **argv) {
    if (argc != 4) {
        fprintf(stderr, "usage: astro-helper LATITUDE LONGITUDE ELEVATION_METRES < epoch_seconds.txt\n");
        return 2;
    }
    char *end = NULL;
    errno = 0;
    double lat = strtod(argv[1], &end);
    if (errno || !end || *end) return 3;
    double lon = strtod(argv[2], &end);
    if (errno || !end || *end) return 3;
    double elevation = strtod(argv[3], &end);
    if (errno || !end || *end) return 3;
    if (lat < -90.0 || lat > 90.0 || lon < -180.0 || lon > 180.0) return 4;

    astro_observer_t observer = Astronomy_MakeObserver(lat, lon, elevation);
    long long epoch;
    while (scanf("%lld", &epoch) == 1) {
        int status = emit_row((int64_t)epoch, observer);
        if (status != 0) {
            fprintf(stderr, "astronomy calculation failed with status %d\n", status);
            return status;
        }
    }
    return 0;
}
