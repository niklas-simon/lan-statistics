import { ActionIcon, Blockquote, Button, Checkbox, Modal, PasswordInput, Stack, TextInput } from "@mantine/core";
import { useDisclosure } from "@mantine/hooks";
import { invoke } from "@tauri-apps/api/core";
import { ChangeEvent, useState } from "react";
import { AlertCircle, Settings as SettingsIcon } from "react-feather";
import { Config } from "../interfaces";

interface SettingsFormProps {
    config: Config,
    setConfig: (c: Config) => void
};

function SettingsForm({config, setConfig}: SettingsFormProps) {
    const [loading, setLoading] = useState(false);
    const [error, setError] = useState<string | null>(null);

    function change<T = string>(k: keyof(Config), t?: (e: ChangeEvent<HTMLInputElement>) => T) {
        return (e: ChangeEvent<HTMLInputElement>) => 
            setConfig({...config, [k]: t ? t(e) : e.currentTarget.value});
    }

    return <form onSubmit={() => {
                setLoading(true);
                invoke<void>("set_config", {config})
                    .then(() => setLoading(false))
                    .catch((e: string) => setError(e));
            }}>
                <Stack gap="md" p="md">
                    {error && <Blockquote color="red" icon={<AlertCircle/>}>
                        {error}
                    </Blockquote>}
                    <TextInput label="ID"
                        readOnly
                        disabled={loading}
                        value={config.id}/>
                    <TextInput
                        label="Name"
                        disabled={loading}
                        value={config.name || ""}
                        onChange={change("name")}/>
                    <TextInput
                        label="Remote-URL"
                        disabled={loading}
                        value={config.remote}
                        onChange={change("remote")}/>
                    <PasswordInput
                        label="Passwort"
                        disabled={loading}
                        value={config.password || ""}
                        onChange={change("password")}/>
                    <Checkbox
                        label="Autostart"
                        disabled={loading}
                        checked={config.autostart}
                        onChange={change("autostart", e => e.currentTarget.checked)}/>
                    <Button type="submit" loading={loading}>Speichern</Button>
                </Stack>
            </form>
}

export default function Settings(props: SettingsFormProps) {
    const [opened, { open, close }] = useDisclosure(false);

    return <>
        <ActionIcon onClick={open} color={opened ? "blue" : "gray"} variant="subtle" size="xl">
            <SettingsIcon />
        </ActionIcon>
        <Modal opened={opened} onClose={close} title="Settings">
            <SettingsForm {...props} />
        </Modal>
    </>
}