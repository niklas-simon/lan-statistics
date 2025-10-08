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