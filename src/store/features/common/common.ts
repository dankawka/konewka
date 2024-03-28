import { createSlice, createAction } from "@reduxjs/toolkit";
import { RootState } from "../../store";
import { ImportConfigurationPayload } from "../../../common/types";

type CommonState = {
  configurationPathToImport: string;
};

const initialState: CommonState = {
  configurationPathToImport: "",
};

export const commonSlice = createSlice({
  name: "common",
  initialState,
  reducers: {
    setConfigurationPathToImport: (state, action) => {
      state.configurationPathToImport = action.payload;
    },
  },
});

export const { setConfigurationPathToImport } = commonSlice.actions;

export const invokeSelectFile = createAction("common/invokeSelectFile");
export const invokeImportConfiguration =
  createAction<ImportConfigurationPayload>("common/invokeImportConfiguration");
export const invokeRemoveConfiguration =
  createAction<string>("common/invokeRemoveConfiguration");
export const invokeNewTunnel = createAction<string>("common/invokeNewTunnel");
export const invokeDisconnectSession = createAction<string>("common/invokeDisconnectSession");
export const invokeConnectSession = createAction<string>("common/invokeConnectSession");


export const getConfigurationPathToImport = (state: RootState) =>
  state.common.configurationPathToImport;

export default commonSlice.reducer;
