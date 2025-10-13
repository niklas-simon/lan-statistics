import { Stack, Text } from "@mantine/core";
import { Config, OthersPlayingEntry } from "../interfaces";
import GameCard from "./GameCard";
import { useMemo } from "react";

export default function GameList({games, config}: {games: OthersPlayingEntry[], config: Config}) {    
    if (games.length === 0) {
        return <Text>(niemand spielt etwas)</Text>
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

    return <Stack flex={1} justify="center" p="md">
        {sorted_games.length > 1 && <Stack flex={2} />}
        <Stack gap="xs">
            <Text>die Mehrheit spielt</Text>
            <GameCard primary game={sorted_games[0]} config={config} />
        </Stack>
        {sorted_games.length > 1 && <Stack gap="xs" flex={3}>
            <Text>es wird ebenfalls gespielt</Text>
            <Stack style={{overflow: "hidden"}}>
                {sorted_games.slice(1).map(game => <GameCard key={game.game.name} game={game} config={config} />)}
            </Stack>
        </Stack>}
    </Stack>
}