# Geometry

Geometry is an engineering model (`Geometry` / `SolidGeometry` / `MeshGeometry` / …), not a file.

`MotorGeometry representedBy ArtifactDigest(format=step)`.

Formats that are **not** classes: STEP, DWG, DXF, SLDPRT, SLDASM, IPT, IAM, STL, glTF.

v1 feature types: Hole, Slot, Pocket, Fillet, Chamfer. Circular/cylindrical/planar/conical are shape **classifications** (properties), not hundreds of subclasses.

Reference geometry: `ReferenceFrame`, `Datum`. Bounding box uses QuantityProperty (length).
