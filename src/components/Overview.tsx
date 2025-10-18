import { Stack } from "@mantine/core";
import { listen } from "@tauri-apps/api/event";
import { useEffect, useState } from "react";
import { Config, OthersPlayingResponse } from "../interfaces";
import { invoke } from "@tauri-apps/api/core";
import GameList from "./GameList";

export default function Overview({config}: {config: Config}) {
    const [others_playing, setOthersPlaying] = useState<OthersPlayingResponse | null>(null);

    useEffect(() => {
        listen<OthersPlayingResponse>("others_playing", e => {
            console.log(e);
            setOthersPlaying(e.payload);
        });

        invoke<OthersPlayingResponse | null>("get_now_playing").then(res => {
            if (!res) {
                return;
            }

            setOthersPlaying(res)
        });
    }, []);

    return <Stack justify="center" align="center" flex={1}>
        <GameList games={others_playing?.active || []} config={config} />
    </Stack>
}