// Git authority for @os20/engineering. The compiler consumes BoundSourceGraph,
// not these tokens. Identity: @os20/engineering#Name
package Engineering {
    private import ScalarValues::*;
    part def EngineeringThing :> Thing;
    part def EngineeringDefinition :> EngineeringThing;
    part def EngineeringInstance :> EngineeringThing;
    part def EngineeringProperty :> EngineeringThing;
    part def QuantityProperty :> EngineeringProperty;
    part def SystemDefinition :> EngineeringDefinition;
    part def SystemInstance :> EngineeringInstance;
    part def PhysicalDefinition :> EngineeringDefinition, PhysicalThing;
    part def PhysicalInstance :> EngineeringInstance, PhysicalThing;
    part def AssemblyDefinition :> PhysicalDefinition;
    part def PartDefinition :> PhysicalDefinition;
    part def DeviceDefinition :> PhysicalDefinition;
    part def BehaviorDefinition :> EngineeringDefinition, BehaviouralThing;
    part def InterfaceDefinition :> EngineeringDefinition;
    part def PortDefinition :> EngineeringDefinition;
    part def PortInstance :> EngineeringInstance;
    part def ConnectionInstance :> EngineeringInstance;
    part def MechanicalPort :> PortDefinition;
    part def ElectricalPort :> PortDefinition;
    part def FluidPort :> PortDefinition;
    part def ThermalPort :> PortDefinition;
    part def DataPort :> PortDefinition;
    part def Configuration :> EngineeringInstance;
    part def Option :> EngineeringThing;
}
