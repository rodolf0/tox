# Unidades: Next Steps — Affine Units & Non-SI Support

## Goal

Extend `Quantity` to support **affine units** (scale + offset, e.g. °F, °C) and **non-SI linear units** (e.g. ft, mile, psi) with automatic display round-tripping, while keeping scalar operations intuitive.

## Core Insight

If a human writes `32 °F * 2`, they expect `64 °F`. This means **scalar operations must happen in the view space** (the unit the user originally expressed the quantity in), not in raw SI space. Quantity-quantity operations still happen in SI space for physical correctness.

## Design

### `DisplayUnit`

Replace the current `display_name: Option<(&str, &str)>` with a richer `DisplayUnit`:

```rust
pub struct DisplayUnit {
    pub symbol: &'static str,
    pub name: &'static str,
    /// SI = display_value * scale + offset
    pub scale: f64,
    pub offset: f64,
}
```

Linear units are affine with `offset = 0.0`. Every `Quantity` stores an optional `DisplayUnit`.

### Value storage

The `value` field **always** stores the raw SI base-unit value:

- `5 ft`   → `value: 1.524` (m), `display_unit: Some(ft{scale: 0.3048, offset: 0})`
- `32 °F`  → `value: 273.15` (K), `display_unit: Some(°F{scale: 5/9, offset: 255.372...})`
- `5 m`    → `value: 5.0`, `display_unit: None` (falls back to SI auto-prefixing)

### Scalar operations (view-space)

| Op | Behaviour |
|----|-----------|
| `f64 * Quantity` | Convert `f64` to SI using the quantity's display unit, store the display unit. |
| `Quantity * f64` | Convert SI value back to view space, multiply, convert back to SI. Keep display unit. |
| `Quantity / f64` | Same as multiply. |
| `Neg` | Same as multiply by `-1.0`. |

Example:
```
(32 °F) * 2
  view = (273.15 - 255.372) / (5/9) = 32.0
  new_view = 64.0
  new_si = 64.0 * (5/9) + 255.372 = 290.928 K
  displays as: "64 °F"
```

For linear units, the math collapses to the same result as SI-space, so nothing breaks.

### Quantity-quantity operations (SI space)

All quantity-quantity operations work on raw SI values. The result's display unit is determined by propagation rules.

**Addition / Subtraction propagation:**
- If exactly one operand is a *point* (`offset != 0`), keep that unit on the result.
- If both operands are *intervals* (`offset == 0`) with the **same** display unit, keep it.
- If both operands are *points* with the **same** display unit:
  - **Addition**: keep the point unit (colloquially weird but mathematically consistent).
  - **Subtraction**: the offset cancels → result is an **interval**. Store with `offset: 0` but keep the scale.
- Otherwise (mixed origins, different units): drop display unit entirely (fall back to raw SI display).

Example:
```
32 °F + 10 K
  si = 273.15 + 10 = 283.15 K
  one point (°F), one interval (K) → keep °F
  display: (283.15 - 255.372) / (5/9) = 50 °F

100 °F - 32 °F
  si = 310.928 - 273.15 = 37.778 K
  same point unit, subtraction → interval
  display: (37.778 - 0) / (5/9) = 68 °F
```

**Multiplication / Division:**
Always drop the display unit. Compound dimensions (`m²`, `Pa`, etc.) have no natural single display unit.

### Display

If `display_unit` is present:
```rust
display_value = (si_value - offset) / scale
write!(f, "{} {}", display_value, symbol)
```

If `display_unit` is `None`, fall back to the current SI logic:
- Named units with metric prefixes (`kg` → `g` for sub-kilogram)
- Dimension string for compound units
- Plain number for dimensionless

### `in_unit(unit: Quantity) -> f64`

Converts the SI value to the target unit's view space:
```rust
(32.0 * °F).in_unit(°C)   // (273.15 - 273.15) / 1.0 = 0.0
(5.0 * ft).in_unit(inch)  // (1.524 - 0) / 0.0254 = 60.0
```

Ignores the source quantity's display metadata entirely.

## Edge Cases & Known Behaviours

1. **`32 °F + 32 °F` = `541.67 °F`**  
   Mathematically consistent (273.15 K + 273.15 K = 546.30 K, converted back), but colloquially weird. This is the cost of treating points as absolute thermodynamic quantities.

2. **`(100 °F - 32 °F) + 32 °F` = `100 °F`**  
   Interval (68 °F) + point (32 °F) = point (100 °F). Correct and intuitive.

3. **`5 ft * 2` = `10 ft`**  
   Linear unit; view-space and SI-space math coincide.

4. **`5 ft + 3 m` = `6.524 m`**  
   Different display units, addition drops to SI display. Result shows as SI, not feet or meters.

5. **Display unit lost on compound ops:**  
   `5 ft * 3 ft` → `4.645 m²`. No auto-inferred "sq ft" display.

6. **`rad` and `sr` as `DisplayUnit`:**  
   Currently they use a fake `display_name` to show `rad` instead of `1`. With `DisplayUnit` they'd become `DisplayUnit{symbol: "rad", scale: 1.0, offset: 0.0}`. Multiplication (`rad * rad`) would drop the unit and display as `1`.

## Proposed Implementation Order

1. Introduce `DisplayUnit`, update `Quantity` struct.
2. Update scalar `Mul`/`Div`/`Neg` to operate in view space.
3. Update `Add`/`Sub` propagation rules.
4. Update `Display` to use `DisplayUnit` when present, fall back to SI logic.
5. Update `in_unit` to ignore source display metadata.
6. Add non-SI constants module (`ft`, `inch`, `mi`, `°C`, `°F`, etc.).
7. Update `rad`/`sr` to use `DisplayUnit` instead of the current hack.
8. Add comprehensive tests for all edge cases above.

## Future Considerations (out of scope for now)

- **Temperature interval literals:** There is no way to write `10 °F` as an interval (delta) directly. Intervals must come from subtraction or use Kelvin. This matches scientific convention but may surprise casual users.
- **Gauge pressure / other offsets:** The same machinery supports `psig`, `barg`, etc., but these would be a new module.
- **Custom user-defined units:** A builder API for `DisplayUnit` would let users register their own units at runtime. Not needed yet.
