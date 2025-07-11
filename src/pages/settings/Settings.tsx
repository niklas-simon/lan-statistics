import { Blockquote, Button, Checkbox, PasswordInput, Stack, TextInput } from "@mantine/core";
import { invoke } from "@tauri-apps/api/core";
import { ChangeEvent, useEffect, useState } from "react";
import { AlertCircle } from "react-feather";

interface Config {
    id: string;
    remote: string;
    name?: string;
    autostart: boolean;
    password?: string;
}

const defaultConfig: Config = {
    id: "",
    remote: "https://lan.pein-gera.de",
    autostart: true
}

function getConfig() {
    return invoke<Config>("get_config");
}

export default function Settings() {
    const [config, setConfig] = useState<Config>(defaultConfig);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState<string | null>(null);

    useEffect(() => {
        getConfig().then(c => {
            setConfig(c);
            setLoading(false);
        }).catch((e: string) => setError(e));
    }, []);

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
                        value={config.name}
                        onChange={change("name")}/>
                    <TextInput
                        label="Remote-URL"
                        disabled={loading}
                        value={config.remote}
                        onChange={change("remote")}/>
                    <PasswordInput
                        label="Passwort"
                        disabled={loading}
                        value={config.password}
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