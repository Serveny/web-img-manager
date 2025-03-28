
export type ChatMessageEvent = { event: string, username: string, msg: string, };


export type ChatMessageRequest = { lobby_id: string, msg: string, };


export type ConfirmationResponse = { is_allowed: boolean, error_msg: string | null, };


export type ConnectEvent = { event: string, session_id: string, };


export type ImageProcessedEvent = { event: string, room_id: number, img_id: number, };


export type LobbyDeletedEvent = { event: string, };


export type RoomDeletedEvent = { event: string, room_id: number, };


export type Success = null;


export type SystemNotificationEvent = { event: string, msg: string, msg_type: string, };


export type UploadRequest = { image: File, };


export type UploadResult = { img_id: number, };

