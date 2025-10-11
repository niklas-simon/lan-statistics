import { AppShell, Group, Image, Loader, Stack, Text, Title } from "@mantine/core";
import "./App.css";
import Settings from "./components/Settings";
import Overview from "./components/Overview";
import { useEffect, useState } from "react";
import { Config, Game } from "./interfaces";
import { invoke } from "@tauri-apps/api/core";
import DataCollectionInfo from "./components/DataCollectionInfo";

function App() {
    const [config, setConfig] = useState<Config | null>(null);
    const [games, setGames] = useState<Game[] | null>(null);

    useEffect(() => {
        invoke<Config>("get_config").then(setConfig);
        invoke<Game[]>("get_games").then(setGames);
    }, []);

    if (!config) {
        return <Stack w="100vw" h="100vh" align="center" justify="center">
            <Loader />
        </Stack>
    }

    return <AppShell header={{height: 80}} footer={{height: 80}}>
        <AppShell.Header p="md">
            <Group justify="space-between">
                <Group>
                    <Image src="icon.png" w="3em" />
                    <Title>LAN Manager</Title>
                </Group>
                <Settings config={config} setConfig={setConfig} />
            </Group>
        </AppShell.Header>
        <AppShell.Main display="flex">
            <Overview config={config} />
        </AppShell.Main>
        <AppShell.Footer p="md">
            <Text>LAN Manager erfasst ausgewählte Prozesse und übermittelt sie an {config.remote}. Es findet keine Verarbeitung durch Dritte statt. <DataCollectionInfo config={config} games={games} /></Text>
        </AppShell.Footer>
    </AppShell>
}

export default App;
