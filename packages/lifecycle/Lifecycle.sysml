package Lifecycle {
    part def ManufacturingPlan :> EngineeringDefinition;
    part def ManufacturingOperation :> EngineeringActivity;
    part def ModelArtifact :> EngineeringThing;
    part def GeometryArtifact :> ModelArtifact;
    // CAD formats (STEP, SLDPRT, …) are artifact.format tokens, not classes.
}
