import { configureStore } from "@reduxjs/toolkit";
import createSagaMiddleware from "redux-saga";
import localConfigsReducer from "./features/local-configs/local-configs";
import commonReducer from "./features/common/common";
import logsReducer from "./features/logs/logs";
import appSaga from "./saga/app-saga";

const sagaMiddleware = createSagaMiddleware();

const store = configureStore({
  reducer: {
    localConfigs: localConfigsReducer,
    common: commonReducer,
    logs: logsReducer
  },
  middleware: (getDefaultMiddleware) =>
    getDefaultMiddleware().concat(sagaMiddleware),
});

// Infer the `RootState` and `AppDispatch` types from the store itself
export type RootState = ReturnType<typeof store.getState>;

sagaMiddleware.run(appSaga);
export default store;
