import {
  AmbientLight,
  AxesHelper,
  BufferAttribute,
  BufferGeometry,
  Color,
  DirectionalLight,
  DoubleSide,
  Fog,
  GridHelper,
  Group,
  LineBasicMaterial,
  LineSegments,
  Mesh,
  MeshPhysicalMaterial,
  PerspectiveCamera,
  Scene,
  SRGBColorSpace,
  WebGLRenderer,
  WireframeGeometry,
} from 'three';
import { OrbitControls } from 'three/addons/controls/OrbitControls.js';

import type { RevolutionMeshModel } from './revolutionMesh.ts';

export interface SceneHandle {
  mount(target: HTMLElement): void;
  updateMesh(model: RevolutionMeshModel | null): void;
}

export async function createSceneHandle(): Promise<SceneHandle> {
  const scene = new Scene();
  scene.background = new Color('#fffaf3');
  scene.fog = new Fog('#fffaf3', 8, 24);

  const camera = new PerspectiveCamera(45, 1, 0.1, 1000);
  camera.position.set(6, 5, 8);
  camera.lookAt(0, 0, 0);

  const renderer = new WebGLRenderer({ antialias: true, alpha: true });
  renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));
  renderer.outputColorSpace = SRGBColorSpace;

  const ambient = new AmbientLight('#fff3e1', 1.5);
  const directional = new DirectionalLight('#fff8ef', 2.2);
  directional.position.set(8, 10, 6);
  const rim = new DirectionalLight('#f2c9b2', 1.1);
  rim.position.set(-5, 4, -7);
  scene.add(ambient, directional, rim);

  const axes = new AxesHelper(3);
  const grid = new GridHelper(10, 10, '#d9c6a7', '#eadfce');
  scene.add(axes, grid);

  const meshGroup = new Group();
  scene.add(meshGroup);

  let mountedTarget: HTMLElement | null = null;
  let controls: InstanceType<typeof OrbitControls> | null = null;

  const animate = () => {
    requestAnimationFrame(animate);
    controls?.update();
    renderer.render(scene, camera);
  };
  animate();

  const resize = () => {
    if (!mountedTarget) {
      return;
    }
    const width = mountedTarget.clientWidth || 640;
    const height = Math.max(320, Math.round(width * 0.68));
    renderer.setSize(width, height, false);
    camera.aspect = width / height;
    camera.updateProjectionMatrix();
  };

  const setMesh = (model: RevolutionMeshModel | null) => {
    meshGroup.clear();
    if (!model || model.positions.length === 0 || model.indices.length === 0) {
      return;
    }

    const geometry = new BufferGeometry();
    geometry.setAttribute('position', new BufferAttribute(model.positions, 3));
    geometry.setAttribute('normal', new BufferAttribute(model.normals, 3));
    geometry.setIndex(new BufferAttribute(model.indices, 1));
    geometry.computeBoundingSphere();

    const material = new MeshPhysicalMaterial({
      color: '#c96f47',
      metalness: 0.08,
      roughness: 0.22,
      clearcoat: 0.35,
      clearcoatRoughness: 0.18,
      reflectivity: 0.45,
      side: DoubleSide,
    });
    const mesh = new Mesh(geometry, material);

    const wireframe = new LineSegments(
      new WireframeGeometry(geometry),
      new LineBasicMaterial({ color: '#5a2d17', transparent: true, opacity: 0.24 }),
    );

    meshGroup.add(mesh, wireframe);

    if (geometry.boundingSphere) {
      const radius = geometry.boundingSphere.radius || 1;
      camera.position.set(radius * 1.8, radius * 1.1, radius * 1.8);
      controls?.target.set(0, 0, 0);
      controls?.update();
    }
  };

  return {
    mount(target) {
      mountedTarget = target;
      target.dataset.scene = 'initialized';
      target.innerHTML = '';
      target.appendChild(renderer.domElement);
      controls = new OrbitControls(camera, renderer.domElement);
      controls.enableDamping = true;
      controls.dampingFactor = 0.07;
      controls.autoRotate = true;
      controls.autoRotateSpeed = 0.8;
      controls.target.set(0, 0, 0);
      resize();
      window.addEventListener('resize', resize);
    },
    updateMesh(model) {
      setMesh(model);
      resize();
    },
  };
}
