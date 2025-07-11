import { Stack, Table, TableTbody, TableTd, TableTh, TableThead, TableTr, TextInput } from "@mantine/core";
import { useDebouncedState } from "@mantine/hooks";
import { listen } from "@tauri-apps/api/event";
import { useEffect, useMemo, useState } from "react";
import { Search } from "react-feather";

interface Process {
    name: string,
    exe?: string,
    cmd: string[],
    cwd?: string
}

export default function Processes() {
    const [processes, setProcesses] = useState<Process[]>([]);
    const [search, setSearch] = useDebouncedState("", 200);

    useEffect(() => {
        listen<Process[]>("processes", e => {
            console.log(e);
            setProcesses(e.payload);
        });
    }, []);

    const filteredProcesses = useMemo(() => search ? processes.filter(p => {
        return p.name.includes(search)
            || p.exe?.includes(search)
            || p.cwd?.includes(search)
            || p.cmd.join(" ").includes(search)
    }) : processes, [processes, search]);

    return <Stack>
        <TextInput defaultValue={search} onChange={e => setSearch(e.currentTarget.value)} leftSection={<Search />} />
        <Table>
            <TableThead>
                <TableTr>
                    <TableTh>
                        Name
                    </TableTh>
                    <TableTh>
                        Executeable
                    </TableTh>
                    <TableTh>
                        Command
                    </TableTh>
                    <TableTh>
                        Working Directory
                    </TableTh>
                </TableTr>
            </TableThead>
            <TableTbody>
                {filteredProcesses.map(p => <TableTr>
                    <TableTd>{p.name}</TableTd>
                    <TableTd>{p.exe || "empty"}</TableTd>
                    <TableTd>{p.cmd.join(" ") || "empty"}</TableTd>
                    <TableTd>{p.cwd || "empty"}</TableTd>
                </TableTr>)}
            </TableTbody>
        </Table>
    </Stack>
}