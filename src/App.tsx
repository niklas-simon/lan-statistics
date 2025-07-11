import { Tabs, TabsList, TabsPanel, TabsTab } from "@mantine/core";
import "./App.css";
import { Info, List, Settings as SettingsIcon } from "react-feather";
import Processes from "./pages/processes/Processes";
import Settings from "./pages/settings/Settings";
import { useState } from "react";
import Overview from "./pages/overview/Overview";

function App() {
    const [tab, setTab] = useState<string | null>("processes");

    return <Tabs value={tab} onChange={setTab}>
        <TabsList>
            <TabsTab value="overview" leftSection={<Info />}>
                Overview
            </TabsTab>
            <TabsTab value="processes" leftSection={<List />}>
                Processes
            </TabsTab>
            <TabsTab value="settings" leftSection={<SettingsIcon />}>
                Settings
            </TabsTab>
        </TabsList>
        <TabsPanel value="overview">
            <Overview />
        </TabsPanel>
        <TabsPanel value="processes">
            <Processes/>
        </TabsPanel>
        <TabsPanel value="settings">
            <Settings />
        </TabsPanel>
    </Tabs>
}

export default App;
