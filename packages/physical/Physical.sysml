package Physical {
    part def Geometry :> EngineeringModel;
    part def SolidGeometry :> Geometry;
    part def GeometricFeature :> GeometricEntity;
    part def HoleFeature :> GeometricFeature;
    part def MaterialDefinition :> EngineeringDefinition;
    part def Metal :> MaterialDefinition;
    part def Battery :> DeviceDefinition;
    part def TractionMotorDefinition :> DeviceDefinition, ElectricMotor;
    attribute def Torque :> Thing; // quantity; dimension M L^2 T^-2 in bound graph
}
