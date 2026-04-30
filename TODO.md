# Fluxcap Missing Features & Roadmap

This document catalogs the natural language processing features, grammar rules, and semantic evaluations present in Facebook's legacy Duckling (`time.clj` / `units.py`) that are currently missing from the `fluxcap` engine.

While `fluxcap` natively supports complex, composed sequence intersections (e.g., "the 3rd Tuesday of November"), it lacks the following syntactic sugar, shorthands, and explicit date-math idioms.

## 1. Named Dates, Holidays & Seasons
*   **Static Holidays:** `"Christmas"`, `"Valentine's Day"`, `"Halloween"`, `"New Year's Eve/Day"`
*   **Dynamic Holidays (Nth weekday of month):** `"Thanksgiving"` (4th Thursday of Nov), `"Memorial Day"` (Last Monday of May), `"Labor Day"`, `"Mother's/Father's Day"`
*   **Seasons:** `"Summer"`, `"Fall"`, `"Winter"`, `"Spring"`
*   **Shorthands:** `"EOD"`, `"EOM"`, `"EOY"` (End of Day/Month/Year)

## 2. Parts of the Day
*   **Spans:** `"morning"`, `"afternoon"`, `"evening"`, `"night"`, `"lunch"`
*   **Modifiers:** `"early morning"`, `"tonight"`, `"after work"`, `"after lunch"`

## 3. Interval Parsing (Ranges & Boundaries)
*   **Explicit Ranges:** `"from 9:30 to 11:00"`, `"between 7 and 8 PM"`, `"July 13-15"`
*   **One-Sided Bounds:** `"until 2pm"`, `"by EOD"`, `"before 11am"`, `"after 2pm"`, `"by the end of next month"`

## 4. Advanced Clock Times & Fractions
*   **Fractions of an Hour:** `"half past 3"`, `"a quarter to noon"`, `"20 past 3pm"`
*   **Precision Modifiers:** `"3pmish"`, `"about 3pm"`, `"exactly 3pm"`, `"3pm sharp"`
*   **Military Time:** `"1523"`
*   **Timezones:** `"4pm CET"`, `"8:00 GMT"`

## 5. Advanced Relative Durations
*   **Fuzzy amounts:** `"in a couple hours"`, `"in a few days"`
*   **Fractional amounts:** `"in half an hour"`, `"for three-quarters of an hour"`

## 6. Corporate / ISO Calendar Shorthands
*   **ISO Weeks:** `"Week 12"`, `"Week 12 of 2018"`
*   **Quarters/Halves:** `"Q1 2018"`, `"H2 2019"`

## 7. Interval Extrema & Intersections
*   **First/Last Shortcuts:** `"first day of next month"`, `"last week of 2018"`
*   **Calculated Dates:** `"Monday before Dec 31st"` (Weekday before/after a specific explicit date)
