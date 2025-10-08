import { AppShell, Group, Title } from "@mantine/core";
import "./App.css";
import Settings from "./components/Settings";
import Overview from "./components/Overview";

function App() {
    return <AppShell header={{height: 80}}>
        <AppShell.Header>
            <Group h="100%" justify="space-between" align="center" p="md">
                <Title>LAN Manager</Title>
                <Settings />
            </Group>
        </AppShell.Header>
        <AppShell.Main display="flex">
            <Overview />
        </AppShell.Main>
    </AppShell>
}

export default App;
