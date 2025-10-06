import { Group, Stack, Text } from "@mantine/core";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useEffect, useState } from "react";

export default function Overview() {
    const [games, setGames] = useState<Record<string, string[]>>({});

    useEffect(() => {
        listen<Record<string, string[]>>("games", e => {
            console.log(e);
            setGames(e.payload);
        });

        invoke<Record<string, string[]>>("get_games").then(setGames);
    }, []);

    return <Stack>
        <Text>Games currently played by everyone</Text>
        <Stack gap={"xs"}>
            {Object.keys(games).map(k => <Group>
                <Text>{k}</Text>
                <Stack>
                    {games[k].map(g => <Text>{g}</Text>)}
                </Stack>
            </Group>)}
            {!Object.keys(games).length && <Text>no game currently played</Text>}
        </Stack>
    </Stack>
}