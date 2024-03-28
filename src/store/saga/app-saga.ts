import { invoke } from "@tauri-apps/api";
import { UnlistenFn, emit, listen } from "@tauri-apps/api/event";
import { call, put, takeLatest } from "redux-saga/effects";
import {
  invokeRemoveConfiguration,
  invokeImportConfiguration,
  invokeSelectFile,
  setConfigurationPathToImport,
  invokeNewTunnel,
  invokeDisconnectSession,
  invokeConnectSession,
} from "../features/common/common";
import { ImportConfigurationPayload } from "../../common/types";
import {
  Config,
  Session,
  initializeConfigs,
  initializeSessions,
} from "../features/local-configs/local-configs";

function* init() {
  const configs: Config[] = yield call(invoke, "get_openvpn3_configs");
  yield put(initializeConfigs(configs));

  const sessions = (yield call(invoke, "get_openvpn3_sessions")) as string[];
  const sessionsAsObjects: Session[] = sessions.map((session) => ({
    path: session,
  }));
  yield put(initializeSessions(sessionsAsObjects));
}

function* registerEvents() {
  const unlisten = (yield call(listen, "log", (event) => {
    console.log(event);
  })) as UnlistenFn;
}

function* handleInvokeSelectFile() {
  const selectedFile: string = yield call(invoke, "select_file");
  yield put(setConfigurationPathToImport(selectedFile));
}

function* handleInvokeImportConfiguration(
  action: ReturnType<typeof invokeImportConfiguration>
) {
  type ImportConfigurationRecord = Record<string, ImportConfigurationPayload>;

  const invokeArgs: ImportConfigurationRecord = {
    payload: action.payload,
  };

  yield call(invoke, "import_openvpn3_config", invokeArgs);
  yield put(setConfigurationPathToImport(""));
  yield init();
}

function* handleInvokeRemoveConfiguration(
  action: ReturnType<typeof invokeRemoveConfiguration>
) {
  type DeleteConfigurationRecord = Record<string, string>;
  const invokeArgs: DeleteConfigurationRecord = {
    payload: action.payload,
  };

  yield call(invoke, "remove_config", invokeArgs);
  yield init();
}

function* handleInvokeNewTunnel(
  action: ReturnType<typeof invokeRemoveConfiguration>
) {
  type DeleteConfigurationRecord = Record<string, string>;
  const invokeArgs: DeleteConfigurationRecord = {
    payload: action.payload,
  };

  yield call(invoke, "new_tunnel", invokeArgs);
  yield init();
}

function* handleInvokeDisconnectSession(
  action: ReturnType<typeof invokeDisconnectSession>
) {
  type DisconnectSessionRecord = Record<string, string>;
  const invokeArgs: DisconnectSessionRecord = {
    payload: action.payload,
  };

  yield call(invoke, "disconnect_session", invokeArgs);
  yield init();
}

function* handleInvokeConnectSession(
  action: ReturnType<typeof invokeConnectSession>
) {
  type ConnectSessionRecord = Record<string, string>;
  const invokeArgs: ConnectSessionRecord = {
    payload: action.payload,
  };

  yield call(invoke, "connect_session", invokeArgs);
  yield init();
}

function* appSaga() {
  yield registerEvents();
  yield init();

  yield takeLatest(invokeSelectFile.type, handleInvokeSelectFile);
  yield takeLatest(
    invokeImportConfiguration.type,
    handleInvokeImportConfiguration
  );
  yield takeLatest(
    invokeRemoveConfiguration.type,
    handleInvokeRemoveConfiguration
  );
  yield takeLatest(invokeNewTunnel.type, handleInvokeNewTunnel);
  yield takeLatest(invokeDisconnectSession.type, handleInvokeDisconnectSession);
  yield takeLatest(invokeConnectSession.type, handleInvokeConnectSession);
}

export default appSaga;
