import { Stack, Text } from "@mantine/core";
import { listen } from "@tauri-apps/api/event";
import { useEffect, useState } from "react";

interface Game {
    name: string,
    label: string
}

export default function Overview() {
    const [games, setGames] = useState<Game[]>([]);

    useEffect(() => {
        listen<Game[]>("now_playing", e => {
            console.log(e);
            setGames(e.payload);
        });
    }, []);

    return <Stack>
        <Text>Now playing</Text>
        <Stack gap={"xs"}>
            {games.map(g => <Text>{g.label}</Text>)}
            {!games.length && <Text>no game currently played</Text>}
        </Stack>
    </Stack>
}