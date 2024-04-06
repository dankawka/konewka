import { PayloadAction, createSlice } from "@reduxjs/toolkit";

export type LastSessionStatus = {
  major_code: number;
  minor_code: number;
  status_message: string;
};

export type LastSessionStatusPayload = LastSessionStatus & { path: string };

export type Config = {
  path: string;
  name: string;
  used_count: number;
};

export type Session = {
  path: string;
  major_code: number;
  minor_code: number;
  status_message: string;
  session_created: number;
};

type LocalConfigsState = {
  configs: Config[];
  sessions: Session[];
  sessionsStatus: {
    [sessionPath: string]: LastSessionStatus;
  };
};

const initialState: LocalConfigsState = {
  configs: [],
  sessions: [],
  sessionsStatus: {},
};

export const localConfigsSlice = createSlice({
  name: "localConfigs",
  initialState,
  reducers: {
    initializeConfigs: (state, action) => {
      state.configs = action.payload;
    },
    initializeSessions: (state, action: PayloadAction<Session[]>) => {
      state.sessions = action.payload;

      for (const session of state.sessions) {
        state.sessionsStatus[session.path] = {
          major_code: session.major_code,
          minor_code: session.minor_code,
          status_message: session.status_message,
        };
      }
    },
    updateSessionStatus: (state, action: PayloadAction<LastSessionStatusPayload>) => {
      state.sessionsStatus[action.payload.path] = action.payload;
    },
  },
});

// Action creators are generated for each case reducer function
export const { initializeConfigs, initializeSessions, updateSessionStatus } =
  localConfigsSlice.actions;

export const getAllConfigs = (state: { localConfigs: LocalConfigsState }) =>
  state.localConfigs.configs;

export const getAllSessions = (state: { localConfigs: LocalConfigsState }) =>
  state.localConfigs.sessions;

export const getSessionsStatus = (state: { localConfigs: LocalConfigsState }) =>
  state.localConfigs.sessionsStatus;

export default localConfigsSlice.reducer;
