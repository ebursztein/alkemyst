export interface LocalService {
  baseUrl: string;
  token: string;
  stop(): Promise<void>;
}
export function startLocalService(options?: {
  database?: string;
  coverage?: boolean;
  history?: boolean;
  analysis?: boolean;
}): Promise<LocalService>;
