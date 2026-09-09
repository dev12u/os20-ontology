package Software {
    part def SoftwareComponent :> EngineeringDefinition;
    part def Firmware :> SoftwareComponent;
    part def EmbeddedSoftware :> SoftwareComponent;
    part def API :> InterfaceDefinition;
}
