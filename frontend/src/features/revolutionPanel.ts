import type { DiskMeshBuildRequest, DiskMeshSnapshot } from '../wasm/engine.ts';
import { hasMeshData } from '../render3d/revolutionMesh.ts';

export function createRevolutionPanel(
  onRun: (request: DiskMeshBuildRequest) => Promise<DiskMeshSnapshot>,
): HTMLElement {
  const section = document.createElement('section');
  section.className = 'panel';
  section.innerHTML = '<h2>Disk Method</h2>';

  const form = document.createElement('div');
  form.className = 'control-grid';
  const start = createNumberControl('Start', 0);
  const end = createNumberControl('End', 3);
  const axialSegments = createNumberControl('Axial Segments', 24, 1);
  const radialSegments = createNumberControl('Radial Segments', 24, 3);
  form.append(start.label, end.label, axialSegments.label, radialSegments.label);

  const button = document.createElement('button');
  button.type = 'button';
  button.textContent = 'Build Disk Mesh';
  button.className = 'panel-button';

  const viewport = document.createElement('div');
  viewport.className = 'scene-frame';
  const sceneNote = document.createElement('p');
  sceneNote.className = 'panel-copy scene-note';
  sceneNote.textContent = '3D scene loads on demand to keep the main bundle lighter.';

  const result = document.createElement('p');
  result.className = 'panel-copy panel-result';
  result.textContent = 'Generate a revolution mesh, its bounds, and summary metadata.';

  let scenePromise: Promise<{ mount(target: HTMLElement): void; updateMesh(model: DiskMeshSnapshot | null): void }> | null = null;

  const ensureScene = async () => {
    if (!scenePromise) {
      scenePromise = import('../render3d/scene.ts').then(async ({ createSceneHandle }) => {
        const handle = await createSceneHandle();
        handle.mount(viewport);
        return handle;
      });
    }
    return scenePromise;
  };

  button.addEventListener('click', () => {
    void (async () => {
      button.disabled = true;
      result.textContent = 'Building disk mesh...';
      try {
        const scene = await ensureScene();
        const snapshot = await onRun({
          start: Number(start.input.value),
          end: Number(end.input.value),
          axialSegments: Number(axialSegments.input.value),
          radialSegments: Number(radialSegments.input.value),
        });
        scene.updateMesh(hasMeshData(snapshot) ? snapshot : null);
        result.textContent = `verts=${snapshot.positions.length / 3} tris=${snapshot.indices.length / 3} max_radius=${snapshot.maxRadius.toFixed(4)} volume=${snapshot.estimatedVolume.toFixed(4)} bounds=[${Array.from(snapshot.boundsMin).map((v) => v.toFixed(2)).join(', ')}] -> [${Array.from(snapshot.boundsMax).map((v) => v.toFixed(2)).join(', ')}]`;
      } catch (error) {
        result.textContent = error instanceof Error ? error.message : String(error);
      } finally {
        button.disabled = false;
      }
    })();
  });

  section.append(form, button, sceneNote, viewport, result);
  return section;
}

function createNumberControl(labelText: string, value: number, min?: number) {
  const input = document.createElement('input');
  input.type = 'number';
  input.value = String(value);
  if (min !== undefined) {
    input.min = String(min);
  }
  return { label: wrapControl(labelText, input), input };
}

function wrapControl(text: string, control: HTMLElement): HTMLLabelElement {
  const label = document.createElement('label');
  const span = document.createElement('span');
  span.textContent = text;
  label.append(span, control);
  return label;
}
