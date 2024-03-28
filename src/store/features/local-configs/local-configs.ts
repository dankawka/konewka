import { createSlice } from "@reduxjs/toolkit";

export type Config = {
  path: string;
  name: string;
  used_count: number;
};

export type Session = {
  path: string;
};

type LocalConfigsState = {
  configs: Config[];
  sessions: Session[];
};

const initialState: LocalConfigsState = {
  configs: [],
  sessions: [],
};

export const localConfigsSlice = createSlice({
  name: "localConfigs",
  initialState,
  reducers: {
    initializeConfigs: (state, action) => {
      state.configs = action.payload;
    },
    initializeSessions: (state, action) => {
      state.sessions = action.payload;
    },
  },
});

// Action creators are generated for each case reducer function
export const { initializeConfigs, initializeSessions } =
  localConfigsSlice.actions;

export const getAllConfigs = (state: { localConfigs: LocalConfigsState }) =>
  state.localConfigs.configs;

export const getAllSessions = (state: { localConfigs: LocalConfigsState }) =>
  state.localConfigs.sessions;

export default localConfigsSlice.reducer;
