import { useDisclosure } from "@mantine/hooks";
import { Config, Game } from "../interfaces";
import { Button, Group, Loader, Modal, Stack, Text } from "@mantine/core";

export default function DataCollectionInfo({config, games, error, reload}: {config: Config, games: Game[] | null, error: string | null, reload: () => void}) {
    const [opened, { open, close }] = useDisclosure(false);

    return <>
        <a href="#" onClick={open}>Weitere Informationen</a>
        <Modal opened={opened} onClose={close} title="Informationen zur Datenerhebung">
            <Text>Diese Anwendung verarbeitet lokal erkannte Prozessnamen und übermittelt diese nach Filterung zwecks Funktionsbereitstellung an {config.remote}. Eine Weitergabe an Dritte oder Drittlandsübermittlung erfolgt nicht.</Text>
            <Text>Rechtsgrundlage: berechtigtes Interesse (Art. 6 Abs. 1 lit. f DSGVO).</Text>
            <Text>Folgende Prozessnamen werden nach Erkennung übermittelt:</Text>
            {games ?
                <ul>
                    {games.map(g => <li key={g.name}>{g.name}</li>)}
                </ul>
            :
                <Stack>
                    {error ? <>
                        <Text c="red">
                            <pre style={{whiteSpace: "pre-wrap"}}>{error}</pre>
                        </Text>
                        <Button onClick={reload}>Erneut versuchen</Button>
                    </>:
                        <Group p="md" justify="center"><Loader /></Group>
                    }
                </Stack>
            }
        </Modal>
    </>
}