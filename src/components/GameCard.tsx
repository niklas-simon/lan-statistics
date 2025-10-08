import { Group, Paper, Text, Title } from "@mantine/core";
import { OthersPlayingEntry } from "../interfaces";

export default function GameCard({game, primary}: {game: OthersPlayingEntry, primary?: boolean}) {
    return <Paper withBorder p="md" style={primary ? {borderColor: "var(--mantine-color-blue-text)"} : undefined}>
        <Title size={primary ? undefined : "md"}>{game.game.label}</Title>
        <Group gap="xs">
            <Text>{game.players
                .map(p => p.name)
                .sort()
                .slice(0, 2)
                .join(", ")}</Text>
            <Text>
                {game.players.length > 2 ? ` +${game.players.length - 2}` : ""}
            </Text>
        </Group>
    </Paper>
}