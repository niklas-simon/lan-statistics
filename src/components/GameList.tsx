import { Stack, Text } from "@mantine/core";
import { OthersPlayingEntry } from "../interfaces";
import GameCard from "./GameCard";
import { useMemo } from "react";

export default function GameList({games}: {games: OthersPlayingEntry[]}) {    
    if (games.length === 0) {
        return <Text>(no one is playing anything)</Text>
    }

    const sorted_games = useMemo(() => {
        return games.sort((a, b) => {
            if (b.players.length !== a.players.length) {
                return b.players.length - a.players.length;
            } else {
                return a.game.label.localeCompare(b.game.label)
            }
        });
    }, [games]);

    return <Stack flex={1}>
        <Stack flex={1} />
        <Stack gap="xs">
            <Text>played by most</Text>
            <GameCard primary game={sorted_games[0]} />
        </Stack>
        <Stack gap="xs" flex={3}>
            <Text>also currently played</Text>
            <Stack style={{overflow: "hidden"}}>
                {sorted_games.slice(1).map(game => <GameCard key={game.game.name} game={game} />)}
            </Stack>
        </Stack>
    </Stack>
}