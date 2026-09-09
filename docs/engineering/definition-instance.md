# Definition vs instance

| Rule | Meaning |
| --- | --- |
| `instanceOf` target | MUST be an `EngineeringDefinition` (or subtype) |
| Definitions | MAY specialize definitions (`TractionMotorDefinition :> DeviceDefinition`) |
| Instances | MUST NOT specialize definitions instead of `instanceOf` (`OS20-E4017`) |
| Package export | `@supplier/motor` **provides** `TractionMotorDefinition` |
| Downstream | `@vehicle/powertrain` depends on the package; `motor01 instanceOf TractionMotorDefinition` |

Package ≠ system. Release ≠ artifact. Configuration ≠ specialization.
