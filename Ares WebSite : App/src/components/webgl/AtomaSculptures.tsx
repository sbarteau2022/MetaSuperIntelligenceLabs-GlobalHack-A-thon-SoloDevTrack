import React, { useRef, useMemo, useState } from 'react';
import { Canvas, useFrame } from '@react-three/fiber';
import * as THREE from 'three';

/**
 * Custom Thick Rectangular-Profile Machined Titanium Ring Geometry
 * Matches QGh3P.jpg: flat machined outer/inner faces, chamfered edges,
 * and a recessed inner rim light channel.
 */
function createMachinedRingGeometry(radius: number, radialThickness: number, axialWidth: number) {
  const shape = new THREE.Shape();
  const rIn = radius - radialThickness / 2;
  const rOut = radius + radialThickness / 2;

  shape.absarc(0, 0, rOut, 0, Math.PI * 2, false);
  const holePath = new THREE.Path();
  holePath.absarc(0, 0, rIn, 0, Math.PI * 2, true);
  shape.holes.push(holePath);

  const geo = new THREE.ExtrudeGeometry(shape, {
    depth: axialWidth,
    bevelEnabled: true,
    bevelThickness: 0.025,
    bevelSize: 0.022,
    bevelSegments: 3,
    curveSegments: 96,
  });
  geo.center();
  return geo;
}

/**
 * Custom Thick Strut Lattice Builder: creates machined rectangular-beam struts
 * along every edge of a polyhedron (Octahedron / Icosahedron) to match the
 * hollow triangular truss core of QGh3P.jpg!
 */
function PolyhedronTrussCage({
  geometry,
  strutRadius = 0.075,
  material,
}: {
  geometry: THREE.BufferGeometry;
  strutRadius?: number;
  material: THREE.Material;
}) {
  const { struts, vertices } = useMemo(() => {
    const edgesGeo = new THREE.EdgesGeometry(geometry, 1);
    const pos = edgesGeo.attributes.position;
    const strutList: Array<{
      position: THREE.Vector3;
      quaternion: THREE.Quaternion;
      length: number;
    }> = [];
    const up = new THREE.Vector3(0, 1, 0);
    const vMap = new Map<string, THREE.Vector3>();

    for (let i = 0; i < pos.count; i += 2) {
      const a = new THREE.Vector3().fromBufferAttribute(pos, i);
      const b = new THREE.Vector3().fromBufferAttribute(pos, i + 1);
      const mid = new THREE.Vector3().addVectors(a, b).multiplyScalar(0.5);
      const dir = new THREE.Vector3().subVectors(b, a);
      const len = dir.length();
      const quat = new THREE.Quaternion().setFromUnitVectors(up, dir.normalize());
      strutList.push({ position: mid, quaternion: quat, length: len });

      [a, b].forEach((v) => {
        const key = `${v.x.toFixed(2)},${v.y.toFixed(2)},${v.z.toFixed(2)}`;
        if (!vMap.has(key)) vMap.set(key, v);
      });
    }

    return {
      struts: strutList,
      vertices: Array.from(vMap.values()),
    };
  }, [geometry]);

  return (
    <group>
      {struts.map((s, idx) => (
        <mesh
          key={idx}
          position={s.position}
          quaternion={s.quaternion}
          material={material}
        >
          {/* 4-sided chamfered square beam matching the machined triangular truss struts */}
          <cylinderGeometry args={[strutRadius, strutRadius, s.length * 1.02, 4, 1]} />
        </mesh>
      ))}
      {vertices.map((v, idx) => (
        <mesh key={`v-${idx}`} position={v} material={material}>
          <octahedronGeometry args={[strutRadius * 1.35, 0]} />
        </mesh>
      ))}
    </group>
  );
}

/**
 * 3D Machined Geodesic Octahedral-Icosahedral Truss Core + 3 Interlocking Gyroscopic Titanium Bands
 * Direct 3D WebGL realization of QGh3P.jpg with dome/gimbal 3D motion & cyan rim light!
 */
function GeodesicGyroscopeLogoMesh({
  variant = 'hero',
  dragRotation,
}: {
  variant?: 'hero' | 'cube' | 'rings' | 'network' | 'holonomy';
  dragRotation: React.MutableRefObject<{ x: number; y: number }>;
}) {
  const rootRef = useRef<THREE.Group>(null);
  const outerCageRef = useRef<THREE.Group>(null);
  const innerCageRef = useRef<THREE.Group>(null);
  const band1Ref = useRef<THREE.Group>(null);
  const band2Ref = useRef<THREE.Group>(null);
  const band3Ref = useRef<THREE.Group>(null);
  const rimLightRef = useRef<THREE.PointLight>(null);

  // Polyhedron geometries for the nested geodesic truss core
  const outerTrussGeo = useMemo(() => new THREE.IcosahedronGeometry(1.18, 0), []);
  const middleOctaGeo = useMemo(() => new THREE.OctahedronGeometry(0.82, 0), []);
  const innerLatticeGeo = useMemo(() => new THREE.IcosahedronGeometry(0.52, 0), []);

  // 3 Thick Rectangular-Cross-Section Gyroscopic Rings
  const ringGeo1 = useMemo(() => createMachinedRingGeometry(1.92, 0.22, 0.24), []);
  const ringGeo2 = useMemo(() => createMachinedRingGeometry(1.76, 0.20, 0.22), []);
  const ringGeo3 = useMemo(() => createMachinedRingGeometry(1.60, 0.18, 0.20), []);

  // Brushed Titanium & Gunmetal Physical Materials
  const brushedTitanium = useMemo(
    () =>
      new THREE.MeshPhysicalMaterial({
        color: new THREE.Color('#cfd6e0'),
        metalness: 0.94,
        roughness: 0.2,
        clearcoat: 0.45,
        clearcoatRoughness: 0.18,
      }),
    []
  );

  const darkGunmetal = useMemo(
    () =>
      new THREE.MeshPhysicalMaterial({
        color: new THREE.Color('#5a6575'),
        metalness: 0.92,
        roughness: 0.28,
        clearcoat: 0.3,
      }),
    []
  );

  const cyanRimGlowMat = useMemo(
    () =>
      new THREE.MeshStandardMaterial({
        color: new THREE.Color('#38bdf8'),
        emissive: new THREE.Color('#0ea5e9'),
        emissiveIntensity: 3.2,
        roughness: 0.1,
        metalness: 0.1,
      }),
    []
  );

  const cubeEdges = useMemo(
    () => new THREE.EdgesGeometry(new THREE.BoxGeometry(3.6, 3.6, 3.6)),
    []
  );

  useFrame((state) => {
    const t = state.clock.getElapsedTime();

    if (rootRef.current) {
      // Smooth 3D dome / gimbal floating motion + interactive drag
      rootRef.current.rotation.y =
        Math.sin(t * 0.35) * 0.24 + dragRotation.current.y;
      rootRef.current.rotation.x =
        Math.cos(t * 0.28) * 0.12 + dragRotation.current.x;
      rootRef.current.position.y = Math.sin(t * 1.2) * 0.06;
    }

    if (outerCageRef.current) {
      outerCageRef.current.rotation.y = t * 0.14;
      outerCageRef.current.rotation.z = Math.sin(t * 0.4) * 0.08;
    }

    if (innerCageRef.current) {
      innerCageRef.current.rotation.y = -t * 0.32;
      innerCageRef.current.rotation.x = t * 0.22;
    }

    // 3 Interlocking Gyroscopic Bands with independent 3D gimbal precession
    if (band1Ref.current) {
      band1Ref.current.rotation.x = 0.68 + Math.sin(t * 0.5) * 0.18;
      band1Ref.current.rotation.y = -0.52 + Math.cos(t * 0.4) * 0.15;
      band1Ref.current.rotation.z = t * 0.28;
    }
    if (band2Ref.current) {
      band2Ref.current.rotation.x = -0.68 + Math.cos(t * 0.45) * 0.18;
      band2Ref.current.rotation.y = 0.52 + Math.sin(t * 0.38) * 0.15;
      band2Ref.current.rotation.z = -t * 0.24;
    }
    if (band3Ref.current) {
      band3Ref.current.rotation.x = Math.PI / 2.15 + Math.sin(t * 0.35) * 0.14;
      band3Ref.current.rotation.y = Math.cos(t * 0.3) * 0.22;
      band3Ref.current.rotation.z = t * 0.18;
    }

    // Traveling cyan-blue arc highlight (upper-right rim glow in QGh3P.jpg)
    if (rimLightRef.current) {
      rimLightRef.current.position.x = 1.45 + Math.sin(t * 1.1) * 0.35;
      rimLightRef.current.position.y = 1.35 + Math.cos(t * 0.9) * 0.25;
      rimLightRef.current.intensity = 6.5 + Math.sin(t * 2.5) * 1.5;
    }
  });

  return (
    <group ref={rootRef}>
      {/* Dynamic Upper-Right Cyan Rim PointLight matching QGh3P.jpg */}
      <pointLight
        ref={rimLightRef}
        position={[1.45, 1.35, 1.1]}
        color="#38bdf8"
        intensity={7}
        distance={4.8}
      />

      {/* Outer Geodesic Triangular Truss Core (Apex pointing up like the Atoma 'A') */}
      <group ref={outerCageRef} rotation={[0.18, 0.4, 0]}>
        <PolyhedronTrussCage
          geometry={outerTrussGeo}
          strutRadius={0.072}
          material={brushedTitanium}
        />
      </group>

      {/* Nested Inner Octahedral & Icosahedral Lattice Truss */}
      <group ref={innerCageRef}>
        <PolyhedronTrussCage
          geometry={middleOctaGeo}
          strutRadius={0.048}
          material={darkGunmetal}
        />
        <PolyhedronTrussCage
          geometry={innerLatticeGeo}
          strutRadius={0.032}
          material={brushedTitanium}
        />
        {/* Innermost luminous N³ core */}
        <mesh material={cyanRimGlowMat}>
          <octahedronGeometry args={[0.22, 0]} />
        </mesh>
      </group>

      {/* Gyroscopic Machined Titanium Band 1 (with cyan inner rim arc) */}
      <group ref={band1Ref}>
        <mesh geometry={ringGeo1} material={brushedTitanium} />
        <mesh rotation={[0, 0, 0.3]} material={cyanRimGlowMat}>
          <torusGeometry args={[1.79, 0.018, 12, 64, Math.PI * 0.65]} />
        </mesh>
      </group>

      {/* Gyroscopic Machined Titanium Band 2 */}
      <group ref={band2Ref}>
        <mesh geometry={ringGeo2} material={brushedTitanium} />
        <mesh rotation={[0, 0, Math.PI]} material={cyanRimGlowMat}>
          <torusGeometry args={[1.64, 0.014, 12, 64, Math.PI * 0.45]} />
        </mesh>
      </group>

      {/* Gyroscopic Machined Titanium Band 3 (Horizontal/Tilted) */}
      <group ref={band3Ref}>
        <mesh geometry={ringGeo3} material={darkGunmetal} />
      </group>

      {/* Optional Architectural Glass Cube Enclosure for 03-meta-hackathon */}
      {variant === 'cube' && (
        <lineSegments geometry={cubeEdges}>
          <lineBasicMaterial color="#389cff" transparent opacity={0.4} />
        </lineSegments>
      )}
    </group>
  );
}

export const AtomaSculptureCanvas: React.FC<{
  variant?: 'hero' | 'cube' | 'rings' | 'network' | 'holonomy';
  height?: string;
  label?: string;
}> = ({
  variant = 'hero',
  height = 'h-[440px]',
  label = 'ATOMA 3D GEODESIC GYROSCOPE · R3F WEBGL',
}) => {
  const dragRotation = useRef({ x: 0, y: 0 });
  const isDragging = useRef(false);
  const lastPos = useRef({ x: 0, y: 0 });
  const [tilt, setTilt] = useState({ x: 0, y: 0 });
  const [renderMode, setRenderMode] = useState<'hybrid-dome' | 'pure-webgl'>('hybrid-dome');

  return (
    <div
      onPointerDown={(e) => {
        isDragging.current = true;
        lastPos.current = { x: e.clientX, y: e.clientY };
      }}
      onPointerMove={(e) => {
        const rect = e.currentTarget.getBoundingClientRect();
        const nx = ((e.clientX - rect.left) / rect.width) * 2 - 1;
        const ny = ((e.clientY - rect.top) / rect.height) * 2 - 1;
        setTilt({ x: -ny * 10, y: nx * 12 });

        if (!isDragging.current) return;
        const dx = e.clientX - lastPos.current.x;
        const dy = e.clientY - lastPos.current.y;
        dragRotation.current.y += dx * 0.008;
        dragRotation.current.x = Math.max(-0.6, Math.min(0.6, dragRotation.current.x + dy * 0.006));
        lastPos.current = { x: e.clientX, y: e.clientY };
      }}
      onPointerUp={() => {
        isDragging.current = false;
      }}
      onPointerLeave={() => {
        isDragging.current = false;
        setTilt({ x: 0, y: 0 });
      }}
      className={`relative w-full ${height} rounded-2xl overflow-hidden bg-black border border-white/10 group cursor-grab active:cursor-grabbing select-none`}
      style={{ perspective: '1100px' }}
    >
      {/* Ambient Cyan-Blue Rim Glow matching QGh3P.jpg */}
      <div
        className="absolute inset-0 pointer-events-none transition-transform duration-300"
        style={{
          background:
            'radial-gradient(circle at 66% 30%, rgba(56, 189, 248, 0.24), rgba(56, 156, 255, 0.08) 38%, transparent 68%)',
          transform: `translate3d(${tilt.y * 0.8}px, ${tilt.x * 0.8}px, 0)`,
        }}
      />

      {/* Mode 1: Studio Raytraced Geodesic Emblem (QGh3P.jpg) + Live 3D WebGL Dome Orbitals */}
      {renderMode === 'hybrid-dome' && (
        <div
          className="absolute inset-0 flex items-center justify-center pointer-events-none transition-transform duration-150 ease-out"
          style={{
            transform: `rotateX(${tilt.x}deg) rotateY(${tilt.y}deg) scale(1.03)`,
            transformStyle: 'preserve-3d',
          }}
        >
          <img
            src="/brand_QGh3P.jpg"
            alt="Atoma 3D Geodesic Gyroscope Logo"
            className="w-full h-full object-contain mix-blend-lighten opacity-95"
            loading="lazy"
            decoding="async"
            fetchPriority="low"
          />
        </div>
      )}

      {/* Live 3D React Three Fiber Geodesic Gyroscope (overlays orbital motion in hybrid mode, or full mesh in pure-webgl mode) */}
      <div className={renderMode === 'hybrid-dome' ? 'absolute inset-0 opacity-45 mix-blend-screen' : 'absolute inset-0'}>
        <Canvas
          camera={{ position: [0, 0.15, 5.2], fov: 40 }}
          dpr={[1, 1.5]}
          gl={{ antialias: false, alpha: true, powerPreference: 'high-performance' }}
        >
          <ambientLight intensity={0.75} />
          <directionalLight position={[5, 8, 6]} intensity={2.6} color="#ffffff" />
          <directionalLight position={[-6, -5, -4]} intensity={1.2} color="#38bdf8" />
          <GeodesicGyroscopeLogoMesh variant={variant} dragRotation={dragRotation} />
        </Canvas>
      </div>

      {/* Mode Switcher Pill (top-left) */}
      <div className="absolute top-3.5 left-4 z-20 flex items-center gap-1 bg-black/75 border border-white/15 rounded-full p-0.5 backdrop-blur-md">
        <button
          type="button"
          onClick={(e) => {
            e.stopPropagation();
            setRenderMode('hybrid-dome');
          }}
          className={`px-2.5 py-1 rounded-full font-mono text-[10px] transition cursor-pointer ${
            renderMode === 'hybrid-dome'
              ? 'bg-[#389cff] text-white font-semibold'
              : 'text-[#9ba5b4] hover:text-white'
          }`}
        >
          3D Dome + Studio Emblem
        </button>
        <button
          type="button"
          onClick={(e) => {
            e.stopPropagation();
            setRenderMode('pure-webgl');
          }}
          className={`px-2.5 py-1 rounded-full font-mono text-[10px] transition cursor-pointer ${
            renderMode === 'pure-webgl'
              ? 'bg-[#389cff] text-white font-semibold'
              : 'text-[#9ba5b4] hover:text-white'
          }`}
        >
          Pure R3F Geodesic Mesh
        </button>
      </div>

      <div className="absolute top-4 right-5 pointer-events-none text-right font-mono text-[10px] tracking-[0.18em] text-[#9ba5b4]/75 space-y-1 uppercase">
        <div>IDEAS</div>
        <div>RUNBOOKS</div>
        <div>AGENTS</div>
        <div className="text-[#38bdf8]">EVIDENCE</div>
        <div>REAL PROGRESS</div>
      </div>

      <div className="absolute bottom-3 left-4 right-4 flex items-center justify-between pointer-events-none font-mono text-[10px] tracking-wider text-[#9ba5b4]/80">
        <span>{label}</span>
        <span className="text-[#38bdf8]">MOVE OR DRAG FOR 3D DOME MOTION ↻</span>
      </div>
    </div>
  );
};
