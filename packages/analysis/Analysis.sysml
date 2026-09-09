package Analysis {
    part def SimulationDefinition :> EngineeringActivity;
    part def SimulationRun :> EngineeringInstance;
    part def SimulationResult :> EngineeringResult, EngineeringEvidence;
    part def ThermalSimulation :> SimulationDefinition;
}
