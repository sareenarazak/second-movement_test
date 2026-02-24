#ifndef CLOCK_WATCHFACE_RS_H
#define CLOCK_WATCHFACE_RS_H
#include <stdbool.h>
#include "watch_common_display.h"
#include "clock_face.h"

void clock_indicate(watch_indicator_t indicator, bool on);
static void clock_indicate_alarm(void);
static void clock_indicate_time_signal(clock_state_t *state);
static void clock_indicate_24h(void);
static void clock_indicate_pm(watch_date_time_t date_time);

void clock_check_battery_periodically(clock_state_t *state, watch_date_time_t date_time);

void clock_toggle_time_signal(clock_state_t *state);
void clock_display_all(watch_date_time_t date_time);

#endif // CLOCK_WATCHFACE_RS_H
