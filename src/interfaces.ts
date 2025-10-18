export interface Game {
    name: string,
    label: string
}

export interface Player {
    id: string,
    name: string
}

export interface OthersPlayingEntry {
    game: Game,
    players: Player[]
};

export interface OthersPlayingResponse {
    active: OthersPlayingEntry[],
    online: number
}

export interface Config {
    id: string;
    remote: string;
    name?: string;
    autostart: boolean;
    password?: string;
}