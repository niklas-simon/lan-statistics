import { Stack } from "@mantine/core";
import { listen } from "@tauri-apps/api/event";
import { useEffect, useState } from "react";
import { OthersPlayingEntry } from "../interfaces";
import { invoke } from "@tauri-apps/api/core";
import GameList from "./GameList";

export default function Overview() {
    const [others_playing, setOthersPlaying] = useState<OthersPlayingEntry[]>([]);

    useEffect(() => {
        listen<OthersPlayingEntry[]>("others_playing", e => {
            console.log(e);
            setOthersPlaying(e.payload);
        });

        invoke<OthersPlayingEntry[] | null>("get_now_playing").then(res => {
            if (!res) {
                return;
            }

            setOthersPlaying(res)
        });
    }, []);

    return <Stack justify="center" align="center" flex={1}>
        <GameList games={others_playing} />
    </Stack>
}