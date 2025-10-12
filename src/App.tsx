import { AppShell, Button, Group, Image, Loader, Stack, Text, Title, Tooltip } from "@mantine/core";
import "./App.css";
import Settings from "./components/Settings";
import Overview from "./components/Overview";
import { useCallback, useEffect, useState } from "react";
import { Config, Game } from "./interfaces";
import { invoke } from "@tauri-apps/api/core";
import DataCollectionInfo from "./components/DataCollectionInfo";
import { listen } from "@tauri-apps/api/event";
import { AlertTriangle } from "react-feather";

function App() {
    const [config, setConfig] = useState<Config | null>(null);
    const [games, setGames] = useState<Game[] | null>(null);
    const [configError, setConfigError] = useState<string | null>(null);
    const [gamesError, setGamesError] = useState<string | null>(null);
    const [pollError, setPollError] = useState<string | null>(null);

    const loadConfig = useCallback(() => {
        setConfigError(null);
        invoke<Config>("get_config").then(setConfig).catch(setConfigError);
    }, []);

    const loadGames = useCallback(() => {
        setGamesError(null);
        invoke<Game[]>("get_games").then(setGames).catch(setGamesError);
    }, []);

    useEffect(() => {
        loadConfig();
        loadGames();

        listen<string | null>("poll_error", e => {
            setPollError(e.payload);
        });
    }, []);

    return <AppShell header={{height: 80}} footer={{height: 80}}>
        <AppShell.Header p="md">
            <Group justify="space-between" align="center">
                <Group>
                    <Image src="icon.png" w="3em" />
                    <Title>LAN Manager</Title>
                </Group>
                <Group align="center">
                    {pollError && <Tooltip label={<pre>{pollError}</pre>}>
                        <Text c="yellow"><AlertTriangle /></Text>
                    </Tooltip>}
                    {config && <Settings config={config} setConfig={setConfig} />}
                </Group>
            </Group>
        </AppShell.Header>
        <AppShell.Main display="flex">
            {config ? 
                <Overview config={config} />
            :
                <Stack w="100vw" h="100vh" align="center" justify="center">
                    {configError ?<>
                        <Text c="red">{configError}</Text>
                        <Button onClick={loadConfig}>Erneut versuchen</Button>
                    </>:
                        <Loader />
                    }
                </Stack>
            }
        </AppShell.Main>
        <AppShell.Footer p="md">
            <Text>LAN Manager erfasst ausgewählte Prozesse und übermittelt sie an {config ? config.remote : "einen Server"}. Es findet keine Verarbeitung durch Dritte statt. {config && <DataCollectionInfo config={config} games={games} error={gamesError} reload={loadGames} />}</Text>
        </AppShell.Footer>
    </AppShell>
}

export default App;
