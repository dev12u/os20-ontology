# Requirements and validation

Classifications: Functional, Performance, Safety, Interface, Environmental, Regulatory, ConstraintRequirement.

Do **not** redefine `RequirementId`, `RequirementFingerprint`, `ValidationDefinition`, `ValidationCase`, `ValidationResult`, `RequirementSatisfaction`.

A requirement may carry a statement **and** a structured binding (property, operator, threshold QuantityValue). Evaluation uses frozen constraints.

`assertsSatisfaction` = allocation/intent. Current satisfaction is derived by the frozen validation engine. Never `requirement.satisfied = true` as ontology state.

Outcomes stay distinct:

| Layer | Values |
| --- | --- |
| ConstraintResult | Satisfied / Violated / Indeterminate |
| ValidationResult | Pass / Fail / Inconclusive (plus NotApplicable / NotEvaluated / EvaluationFailed) |
| RequirementSatisfaction | Satisfied / NotSatisfied / Inconclusive |
