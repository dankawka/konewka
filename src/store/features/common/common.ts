import { createSlice, createAction } from "@reduxjs/toolkit";
import { RootState } from "../../store";
import { ImportConfigurationPayload, Modals } from "../../../common/types";

type CommonState = {
  configurationPathToImport: string;
  notificationsEnabled: boolean;
  currentModal: Modals;
};

const initialState: CommonState = {
  configurationPathToImport: "",
  notificationsEnabled: false,
  currentModal: null,
};

export const commonSlice = createSlice({
  name: "common",
  initialState,
  reducers: {
    setConfigurationPathToImport: (state, action) => {
      state.configurationPathToImport = action.payload;
    },
    setNotificationsEnabled: (state, action) => {
      state.notificationsEnabled = action.payload;
    },
    setCurrentModal: (state, action) => {
      state.currentModal = action.payload;
    },
  },
});

export const {
  setConfigurationPathToImport,
  setNotificationsEnabled,
  setCurrentModal,
} = commonSlice.actions;

export const invokeSelectFile = createAction("common/invokeSelectFile");
export const invokeImportConfiguration =
  createAction<ImportConfigurationPayload>("common/invokeImportConfiguration");
export const invokeRemoveConfiguration = createAction<string>(
  "common/invokeRemoveConfiguration"
);
export const invokeNewTunnel = createAction<string>("common/invokeNewTunnel");
export const invokeDisconnectSession = createAction<string>(
  "common/invokeDisconnectSession"
);
export const invokeConnectSession = createAction<string>(
  "common/invokeConnectSession"
);

export const invokeConfirmExit = createAction("common/invokeConfirmExit");

export const getConfigurationPathToImport = (state: RootState) =>
  state.common.configurationPathToImport;

export const getCurrentModal = (state: RootState) => state.common.currentModal;

export default commonSlice.reducer;
