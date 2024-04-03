import { invoke } from "@tauri-apps/api";
import { listen } from "@tauri-apps/api/event";
import {
  isPermissionGranted,
  requestPermission,
  sendNotification,
} from "@tauri-apps/api/notification";
import { call, fork, put, take, takeLatest } from "redux-saga/effects";
import { channel } from "redux-saga";
import {
  invokeRemoveConfiguration,
  invokeImportConfiguration,
  invokeSelectFile,
  setConfigurationPathToImport,
  invokeNewTunnel,
  invokeDisconnectSession,
  invokeConnectSession,
  setNotificationsEnabled,
  setCurrentModal,
  invokeConfirmExit,
} from "../features/common/common";
import { FromMainAction, ImportConfigurationPayload } from "../../common/types";
import {
  Config,
  Session,
  initializeConfigs,
  initializeSessions,
} from "../features/local-configs/local-configs";
import { Log, addLog } from "../features/logs/logs";

const logsChannel = channel();
const fromMainChannel = channel<FromMainAction>();

const checkNotifPermission = async () => {
  const permissionGranted = await isPermissionGranted();
  if (!permissionGranted) {
    const permission = await requestPermission();
    return permission === "granted";
  }
  return true;
};

function* processLog(log: Log) {
  if (log.member === "StatusChange" && log.second_flag === 7) {
    sendNotification({
      title: "Konewka",
      body: "Connected to VPN!",
    });
  }

  if (log.member === "StatusChange" && log.second_flag === 9) {
    sendNotification({
      title: "Konewka",
      body: "VPN disconnected!",
    });
  }
}

function* init() {
  const configs: Config[] = yield call(invoke, "get_openvpn3_configs");
  yield put(initializeConfigs(configs));

  const sessions = (yield call(invoke, "get_openvpn3_sessions")) as Session[];
  yield put(initializeSessions(sessions));

  for (const session of sessions) {
    const firstLog: Log = {
      member: "StatusChange",
      first_flag: session.major_code,
      second_flag: session.minor_code,
      message: session.status_message,
    };

    yield put(addLog(firstLog));
  }

  const permissionGranted: boolean = yield call(checkNotifPermission);
  yield put(setNotificationsEnabled(permissionGranted));
}

function* registerEvents() {
  yield call(listen<Log>, "log", (event) => {
    logsChannel.put(event.payload);
  });

  yield call(listen, "exit_confirmation", () => {
    fromMainChannel.put({
      type: "exit_confirmation",
      data: null,
    });
  });
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

function* watchLogs() {
  while (true) {
    const log: Log = yield take(logsChannel);
    yield put(addLog(log));
    yield processLog(log);
  }
}

function* watchActions() {
  while (true) {
    const action: FromMainAction = yield take(fromMainChannel);

    if (action.type === "exit_confirmation") {
      yield put(setCurrentModal("exit_confirmation"));
    }
  }
}

function* handleInvokeConfirmExit() {
  yield call(invoke, "exit_app");
}

function* appSaga() {
  yield fork(watchLogs);
  yield fork(watchActions);

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
  yield takeLatest(invokeConfirmExit.type, handleInvokeConfirmExit);
}

export default appSaga;
