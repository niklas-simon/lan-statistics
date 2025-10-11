import { Card, Group, Image, Stack, Text, Title } from "@mantine/core";
import { Config, OthersPlayingEntry } from "../interfaces";

export default function GameCard({game, primary, config}: {game: OthersPlayingEntry, primary?: boolean, config: Config}) {
    return <Card withBorder padding={0} style={primary ? {borderColor: "var(--mantine-color-blue-text)"} : undefined}>
        <Group gap={0}>
            <Image h={primary ? "6em" : "5em"} src={`${config.remote}/api/v1/games/${game.game.name}/icon`} />
            <Stack flex={1} gap={0} m="md">
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
            </Stack>
        </Group>
    </Card>
}