import { useState } from 'react'
import { ScreenNavigator } from './components/ScreenNavigator'
import { ChatPanel } from './components/ChatPanel'
import { Screen01Home } from './screens/Screen01Home'
import { Screen02ToolsMCP } from './screens/Screen02ToolsMCP'
import { Screen03MetaHackathon } from './screens/Screen03MetaHackathon'
import { Screen04ResearchLab } from './screens/Screen04ResearchLab'
import { Screen05Contact } from './screens/Screen05Contact'
import { Screen06SignUp } from './screens/Screen06SignUp'
import { Screen07Downloads } from './screens/Screen07Downloads'
import { Screen08IDEChat } from './screens/Screen08IDEChat'
import { Screen09SwarmHolonomy } from './screens/Screen09SwarmHolonomy'
import { Screen10MindContextGraph } from './screens/Screen10MindContextGraph'
import { Screen11ResearchHubReader } from './screens/Screen11ResearchHubReader'

const screens = [
  <Screen01Home key="home" />,
  <Screen02ToolsMCP key="tools-mcp" />,
  <Screen03MetaHackathon key="meta-hackathon" />,
  <Screen04ResearchLab key="research-lab" />,
  <Screen05Contact key="contact" />,
  <Screen06SignUp key="sign-up" />,
  <Screen07Downloads key="downloads" />,
  <Screen08IDEChat key="ide-chat" />,
  <Screen09SwarmHolonomy key="swarm-holonomy" />,
  <Screen10MindContextGraph key="mind-context-graph" />,
  <Screen11ResearchHubReader key="research-hub-reader" />,
]

function App() {
  return (
    <div style={{ height: '100vh', width: '100vw', overflow: 'hidden' }}>
      <ScreenNavigator screens={screens} />
    </div>
  )
}

export default App
