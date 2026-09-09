package Assurance {
    part def RequirementDefinition :> EngineeringDefinition;
    part def PerformanceRequirement :> RequirementDefinition;
    part def TestDefinition :> VerificationActivity;
    part def TestExecution :> EngineeringInstance;
    part def TestResult :> EngineeringResult, EngineeringEvidence;
}
