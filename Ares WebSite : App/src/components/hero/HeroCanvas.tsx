import { useEffect, useMemo, useRef } from 'react'
import { Canvas, useFrame } from '@react-three/fiber'
import * as THREE from 'three'
import { useReducedMotion } from '../../hooks/useReducedMotion'

const NODE_COUNT = 14
const SHELL_RADIUS = 2.2

function fibonacciSphere(count: number, radius: number): THREE.Vector3[] {
  const points: THREE.Vector3[] = []
  const goldenAngle = Math.PI * (3 - Math.sqrt(5))
  for (let i = 0; i < count; i++) {
    const y = 1 - (i / (count - 1)) * 2
    const r = Math.sqrt(Math.max(0, 1 - y * y))
    const theta = goldenAngle * i
    points.push(
      new THREE.Vector3(Math.cos(theta) * r * radius, y * radius, Math.sin(theta) * r * radius)
    )
  }
  return points
}

function Scene({ reduced }: { reduced: boolean }) {
  const groupRef = useRef<THREE.Group>(null)
  const torusAmberRef = useRef<THREE.Mesh>(null)
  const torusCyanRef = useRef<THREE.Mesh>(null)
  const nodeRefs = useRef<Array<THREE.Mesh | null>>([])
  const pointer = useRef({ x: 0, y: 0 })

  const nodes = useMemo(() => fibonacciSphere(NODE_COUNT, SHELL_RADIUS), [])

  const linePositions = useMemo(() => {
    const arr = new Float32Array(NODE_COUNT * 2 * 3)
    nodes.forEach((p, i) => {
      arr[i * 6 + 0] = 0
      arr[i * 6 + 1] = 0
      arr[i * 6 + 2] = 0
      arr[i * 6 + 3] = p.x
      arr[i * 6 + 4] = p.y
      arr[i * 6 + 5] = p.z
    })
    return arr
  }, [nodes])

  const lineGeometry = useMemo(() => {
    const geo = new THREE.BufferGeometry()
    geo.setAttribute('position', new THREE.BufferAttribute(linePositions, 3))
    return geo
  }, [linePositions])

  useEffect(() => {
    const onMove = (e: MouseEvent) => {
      pointer.current.x = (e.clientX / window.innerWidth) * 2 - 1
      pointer.current.y = -((e.clientY / window.innerHeight) * 2 - 1)
    }
    window.addEventListener('mousemove', onMove)
    return () => window.removeEventListener('mousemove', onMove)
  }, [])

  useEffect(() => {
    return () => {
      lineGeometry.dispose()
    }
  }, [lineGeometry])

  useFrame((state, delta) => {
    const t = state.clock.elapsedTime
    const cam = state.camera

    if (!reduced) {
      if (groupRef.current) {
        groupRef.current.rotation.y += 0.0016
        groupRef.current.rotation.x = THREE.MathUtils.lerp(
          groupRef.current.rotation.x,
          pointer.current.y * 0.12,
          0.03
        )
      }
      if (torusAmberRef.current) {
        torusAmberRef.current.rotation.z -= 0.0022
      }
      if (torusCyanRef.current) {
        torusCyanRef.current.rotation.z += 0.0014
      }
      nodeRefs.current.forEach((mesh, i) => {
        if (!mesh) return
        const s = 1 + Math.sin(t * 2 + i * 0.9) * 0.25
        mesh.scale.setScalar(s)
      })
    }

    const targetX = reduced ? 0 : pointer.current.x * 0.4
    const targetY = reduced ? 0 : pointer.current.y * 0.4
    const k = reduced ? 1 : Math.min(1, delta * 2.5)
    cam.position.x = THREE.MathUtils.lerp(cam.position.x, targetX, k)
    cam.position.y = THREE.MathUtils.lerp(cam.position.y, targetY, k)
    cam.lookAt(0, 0, 0)
  })

  return (
    <group ref={groupRef}>
      {/* Icosahedron wireframe shell */}
      <mesh>
        <icosahedronGeometry args={[2.2, 1]} />
        <meshBasicMaterial color="#22d3ee" wireframe transparent opacity={0.35} />
      </mesh>

      {/* Inner dark core */}
      <mesh>
        <sphereGeometry args={[0.55, 32, 32]} />
        <meshStandardMaterial color="#060a10" roughness={0.4} metalness={0.2} />
      </mesh>

      {/* Amber toroidal ring */}
      <mesh ref={torusAmberRef} rotation={[Math.PI / 2.4, 0.2, 0]}>
        <torusGeometry args={[3.1, 0.035, 16, 128]} />
        <meshBasicMaterial color="#fbbf24" transparent opacity={0.7} />
      </mesh>

      {/* Thin cyan torus, tilted */}
      <mesh ref={torusCyanRef} rotation={[Math.PI / 1.8, -0.35, 0.4]}>
        <torusGeometry args={[3.6, 0.015, 12, 128]} />
        <meshBasicMaterial color="#22d3ee" transparent opacity={0.5} />
      </mesh>

      {/* Orbiting nodes */}
      {nodes.map((p, i) => (
        <mesh
          key={i}
          position={[p.x, p.y, p.z]}
          ref={(m) => {
            nodeRefs.current[i] = m
          }}
        >
          <sphereGeometry args={[0.07, 16, 16]} />
          <meshStandardMaterial color="#22d3ee" emissive="#22d3ee" emissiveIntensity={1.6} />
        </mesh>
      ))}

      {/* Core-to-node connection lines */}
      <lineSegments geometry={lineGeometry}>
        <lineBasicMaterial color="#22d3ee" transparent opacity={0.25} />
      </lineSegments>

      <ambientLight intensity={0.7} />
      <pointLight position={[5, 5, 5]} intensity={1.2} color="#22d3ee" />
    </group>
  )
}

export default function HeroCanvas() {
  const reduced = useReducedMotion()

  return (
    <Canvas
      dpr={[1, 1.5]}
      camera={{ position: [0, 0, 7] }}
      gl={{ antialias: false, alpha: true, powerPreference: 'high-performance' }}
      frameloop={reduced ? 'demand' : 'always'}
    >
      <Scene reduced={reduced} />
    </Canvas>
  )
}
