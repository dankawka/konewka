import { Badge, Code, Flex } from "@chakra-ui/react";
import { useSelector } from "react-redux";
import { getLogs } from "../store/features/logs/logs";
import { useEffect, useRef } from "react";
import { useVirtualizer } from "@tanstack/react-virtual";

type LowRowProps = {
  member: string;
  firstCode: number;
  secondCode: number;
  message: string;

  dataIndex: number;
  setRef: (element: HTMLDivElement) => void;
};

type MapStructure = {
  [key: string]: {
    [key: number]: string;
  };
};

const getMember = (member: string) => {
  switch (member) {
    case "Log":
      return "yellow";
    case "StatusChange":
      return "green";
    default:
      return "grey";
  }
};

const getFirstCodeName = (member: string, firstCode: number) => {
  const map: MapStructure = {
    Log: {
      1: "MASTERPROC",
      2: "CONFIGMGR",
      3: "SESSIONMGR",
      4: "BACKENDSTART",
      5: "LOGGER",
      6: "BACKENDPROC",
      7: "CLIENT",
    },
    StatusChange: {
      1: "UNSET",
      2: "CONFIG",
      3: "CONNECTION",
      4: "SESSION",
      5: "PKCS11",
      6: "PROCESS",
    },
  };

  if (!map[member] || !map[member][firstCode]) {
    return "UNKNOWN";
  }

  return map[member][firstCode];
};

const getSecondCodeName = (method: string, secondCode: number) => {
  const map: MapStructure = {
    Log: {
      1: "DEBUG",
      2: "VERB2",
      3: "VERB1",
      4: "INFO",
      5: "WARNING",
      6: "ERROR",
      7: "CRITICAL",
      8: "FATAL",
    },
    StatusChange: {
      1: "CFG_ERROR",
      2: "CFG_OK",
      3: "CFG_INLINE_MISSING",
      4: "CFG_REQUIRE_USER",

      5: "CONN_INIT",
      6: "CONN_CONNECTING",
      7: "CONN_CONNECTED",
      8: "CONN_DISCONNECTING",
      9: "CONN_DISCONNECTED",
      10: "CONN_FAILED",
      11: "CONN_AUTH_FAILED",
      12: "CONN_RECONNECTING",
      13: "CONN_PAUSING",
      14: "CONN_PAUSED",
      15: "CONN_RESUMING",
      16: "CONN_DONE",

      17: "SESS_NEW",
      18: "SESS_BACKEND_COMPLETED",
      19: "SESS_REMOVED",
      20: "SESS_AUTH_USERPASS",
      21: "SESS_AUTH_CHALLENGE",
      22: "SESS_AUTH_URL",

      23: "PKCS11_SIGN",
      24: "PKCS11_ENCRYPT",
      25: "PKCS11_DECRYPT",
      26: "PKCS11_VERIFY",

      27: "PROC_STARTED",
      28: "PROC_STOPPED",
      29: "PROC_KILLED",
    },
  };

  if (!map[method] || !map[method][secondCode]) {
    return "UNKNOWN";
  }

  return map[method][secondCode];
};

const LogRow = (props: LowRowProps) => {
  return (
    <Flex
      ref={props.setRef}
      data-index={props.dataIndex}
      gap="2px"
      paddingBottom={"2px"}
    >
      <Code w={"100%"}>
        <Flex alignItems={"center"} justifyContent={"space-between"}>
          {props.message || "<no message>"}
          <Flex gap="2px" alignSelf={"start"}>
            <Badge colorScheme={getMember(props.member)}>{props.member}</Badge>
            <Badge colorScheme="purple">
              {getFirstCodeName(props.member, props.firstCode)}
            </Badge>
            <Badge colorScheme="teal">
              {getSecondCodeName(props.member, props.secondCode)}
            </Badge>
          </Flex>
        </Flex>
      </Code>
    </Flex>
  );
};

export const LogsContainer = () => {
  const parentRef = useRef<HTMLDivElement>(null);

  const logs = useSelector(getLogs);

  const count = logs.length;
  const virtualizer = useVirtualizer({
    count,
    getScrollElement: () => parentRef.current,
    estimateSize: () => 21,
  });

  const items = virtualizer.getVirtualItems();

  useEffect(() => {
    if (count > 0) {
      virtualizer.scrollToIndex(count - 1);
    }
  }, [count]);

  return (
    <div
      ref={parentRef}
      style={{ overflowY: "auto", height: "100%", contain: "strict" }}
    >
      <div
        style={{
          height: virtualizer.getTotalSize(),
          width: "100%",
          position: "relative",
        }}
      >
        <div
          style={{
            position: "absolute",
            top: 0,
            left: 0,
            width: "100%",
            transform: `translateY(${items[0]?.start ?? 0}px)`,
          }}
        >
          {items.map((item) => (
            <LogRow
              dataIndex={item.index}
              setRef={virtualizer.measureElement}
              key={item.index}
              member={logs[item.index].member}
              firstCode={logs[item.index].first_flag}
              secondCode={logs[item.index].second_flag}
              message={logs[item.index].message}
            />
          ))}
        </div>
      </div>
    </div>
  );
};
