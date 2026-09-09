# Interfaces, ports, connections

First-class: `InterfaceDefinition`, `PortDefinition`/`PortInstance`, `ConnectionDefinition`/`ConnectionInstance`.

Port kinds: Mechanical, Electrical, Fluid, Thermal, Data.

Flow kinds on the port: energy / material / information (plus `flowKind` token). Direction `in`/`out`/`inout` is a port property — not `electricalOut` predicates.

Compatibility (not names): kind, complementary direction, flow kind, quantity kinds. `OS20-E4019`.

`connects` is authored from Connection → Port (cardinality 2). `connectedTo` is derived.
