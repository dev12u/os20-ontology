# Core ontology

Engineering entities specialize **`Thing`** (`@os20/core#Thing`) unless KerML/SysML requires another language root.

Evaluated lattice (not a blind copy of every proposal):

```text
Thing
 ├── AbstractThing
 ├── PhysicalThing     (+ mixin MassBearing → optional mass)
 ├── InformationThing
 ├── BehaviouralThing
 └── UnallocatedThing
```

`PhysicalThing` may declare optional position, orientation, geometry, material. Not every `Thing` is physical.

`Unallocated<T>` is a **modeling convention**, not a parser keyword: classifier `Unallocated{T}` specializes `T` and `UnallocatedThing`. It is a real classifier. It is not NULL, missing, or `Undefined`.

Absence kinds stay distinct: missing value, null, Nothing, Undefined, Unallocated.

KerML scalars: `@omg/kerml#Boolean|Integer|Real|String`.

Physics (`@os20/physics`) is an extension point: Mass, Force, Energy, Power, Temperature, Pressure, Time, Length as `QuantityType` with dimension tokens (`kg`/`m`/`s`/`N`/`W`/`Pa` representable). No physics solver in Prompt 1.

Electrical (`@os20/electrical`) imports physics. `ElectricMotor` specializes `PhysicalThing` and `BehaviouralThing`; `ServoMotor` specializes `ElectricMotor`. `ElectricMotor` participates in Structure, Electrical, Behaviour, Simulation, Manufacturing.

Standard library (`PackageRole::StandardLibrary`) ≠ OS20 ontology packages (`PackageRole::Ontology`).
