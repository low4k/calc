export interface RevolutionMeshModel {
  positions: Float32Array;
  normals: Float32Array;
  indices: Uint32Array;
}

export function hasMeshData(model: RevolutionMeshModel): boolean {
  return model.positions.length > 0 && model.indices.length > 0;
}
