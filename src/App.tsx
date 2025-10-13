import { Button, Divider, Group, Image, Loader, Stack, Text, Title, Tooltip } from "@mantine/core";
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

    return <Stack gap={0} w="100vw" h="100vh" style={{overflow: "hidden"}}>
        <Stack p="md">
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
        </Stack>
        <Divider/>
        <Stack flex={1} mih={0}>
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
        </Stack>
        <Divider/>
        <Stack p="md" style={{
            backgroundColor: "color-mix(in srgb, var(--mantine-color-body), transparent 30%)",
            backdropFilter: "blur(5px)"
        }}>
            <Text>LAN Manager erfasst ausgewählte Prozesse und übermittelt sie an {config ? config.remote : "einen Server"}.</Text>
            <Text>Es findet keine Verarbeitung durch Dritte statt. {config && <DataCollectionInfo config={config} games={games} error={gamesError} reload={loadGames} />}</Text>
        </Stack>
    </Stack>
}

export default App;
