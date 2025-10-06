import { Blockquote, Button, Checkbox, Loader, Stack, TextInput } from "@mantine/core";
import { invoke } from "@tauri-apps/api/core";
import { ChangeEvent, useEffect, useState } from "react";
import { AlertCircle } from "react-feather";

interface Config {
    autostart: boolean;
    advanced: boolean;
    remote_url: string;
    remote_db: string;
}

function getConfig() {
    return invoke<Config>("get_config");
}

export default function Settings() {
    const [config, setConfig] = useState<Config | null>(null);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState<string | null>(null);

    useEffect(() => {
        getConfig().then(c => {
            setConfig(c);
            setLoading(false);
        }).catch((e: string) => setError(e));
    }, []);

    function change<T = string>(k: keyof(Config), t?: (e: ChangeEvent<HTMLInputElement>) => T) {
        return (e: ChangeEvent<HTMLInputElement>) => {   
            if (!config) {
                return;
            }

            setConfig({...config, [k]: t ? t(e) : e.currentTarget.value});
        }
    }

    if (!config) {
        return <Loader />
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
                    <Checkbox
                        label="Autostart"
                        disabled={loading}
                        checked={config.autostart}
                        onChange={change("autostart", e => e.currentTarget.checked)}/>
                    <Checkbox
                        label="Erweitert"
                        disabled={loading}
                        checked={config.advanced || false}
                        onChange={change("advanced", e => e.currentTarget.checked)}/>
                    {config.advanced && <>
                        <TextInput
                            label="Remote-URL"
                            disabled={loading}
                            value={config.remote_url}
                            onChange={change("remote_url")}/>
                        <TextInput
                            label="Remote-DB"
                            disabled={loading}
                            value={config.remote_db}
                            onChange={change("remote_db")}/>
                    </>}
                    <Button type="submit" loading={loading}>Speichern</Button>
                </Stack>
            </form>
}