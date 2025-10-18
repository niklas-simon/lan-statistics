import { Stack, Text } from "@mantine/core";
import { Config, OthersPlayingEntry } from "../interfaces";
import GameCard from "./GameCard";

export default function GameList({games, config}: {games: OthersPlayingEntry[], config: Config}) {    
    if (games.length === 0) {
        return <Text>(niemand spielt etwas)</Text>
    }

    return <Stack flex={1} justify="center" p="md">
        {games.length > 1 && <Stack flex={2} />}
        <Stack gap="xs">
            <Text>die Mehrheit spielt</Text>
            <GameCard primary game={games[0]} config={config} />
        </Stack>
        {games.length > 1 && <Stack gap="xs" flex={3}>
            <Text>es wird ebenfalls gespielt</Text>
            <Stack style={{overflow: "hidden"}}>
                {games.slice(1).map(game => <GameCard key={game.game.name} game={game} config={config} />)}
            </Stack>
        </Stack>}
    </Stack>
}