import {
  Button,
  Input,
  Checkbox,
  FormControl,
  FormLabel,
  FormErrorMessage,
  Stack,
  IconButton,
  HStack,
  Table,
  TableCaption,
  TableContainer,
  Tbody,
  Td,
  Th,
  Thead,
  Tr,
  Flex,
  Box,
  Text,
  Badge,
} from "@chakra-ui/react";
import { useDispatch, useSelector } from "react-redux";
import { SubmitHandler, useForm } from "react-hook-form";
import { AddIcon, DeleteIcon, LinkIcon } from "@chakra-ui/icons";
import {
  getConfigurationPathToImport,
  invokeRemoveConfiguration,
  invokeImportConfiguration,
  invokeSelectFile,
  invokeNewTunnel,
  invokeDisconnectSession,
  invokeConnectSession,
} from "./store/features/common/common";
import { useEffect } from "react";
import {
  LastSessionStatus,
  getAllConfigs,
  getAllSessions,
  getSessionsStatus,
} from "./store/features/local-configs/local-configs";
import { ImportConfigurationPayload } from "./common/types";
import { LogsContainer } from "./components/Logs";

const ImportConfigurationForm = () => {
  const dispatch = useDispatch();
  const configurationPathToImport = useSelector(getConfigurationPathToImport);
  const {
    register,
    handleSubmit,
    setValue,
    formState: { errors },
  } = useForm<ImportConfigurationPayload>();

  useEffect(() => {
    setValue("configFile", configurationPathToImport);
  }, [configurationPathToImport]);

  const onSubmit: SubmitHandler<ImportConfigurationPayload> = (data) =>
    dispatch(invokeImportConfiguration(data));

  return (
    <form onSubmit={handleSubmit(onSubmit)}>
      <Stack spacing={4}>
        <FormControl isInvalid={Object.keys(errors).length > 0}>
          <FormLabel>Configuration name</FormLabel>
          <Input {...register("configName", { required: true })} />
          {errors.configName && (
            <FormErrorMessage>Configuration name is required.</FormErrorMessage>
          )}
        </FormControl>

        <Checkbox {...register("singleUse")}>Single use?</Checkbox>
        <Checkbox {...register("persistent")}>Persistent?</Checkbox>

        <FormControl isInvalid={Object.keys(errors).length > 0}>
          <FormLabel>Configuration file</FormLabel>
          <HStack>
            <IconButton
              colorScheme="blue"
              aria-label="Choose file"
              icon={<AddIcon />}
              onClick={() => {
                dispatch(invokeSelectFile());
              }}
            />
            <Input disabled {...register("configFile", { required: true })} />
          </HStack>
          {errors.configFile && (
            <FormErrorMessage>Configuration file is required.</FormErrorMessage>
          )}
        </FormControl>

        <Button type="submit">Import</Button>
      </Stack>
    </form>
  );
};

const ConfigurationsList = () => {
  const configs = useSelector(getAllConfigs);
  const dispatch = useDispatch();

  return (
    <TableContainer>
      <Table layout={"fixed"} size="sm" variant="simple">
        <TableCaption>Imported configurations</TableCaption>
        <Thead>
          <Tr>
            <Th w={350}>Path</Th>
            <Th>Name</Th>
            <Th>Use count</Th>
            <Th w={100}>Actions</Th>
          </Tr>
        </Thead>
        <Tbody>
          {configs.map((config) => (
            <Tr key={config.path}>
              <Td>
                <Text
                  overflow="hidden"
                  textOverflow={"ellipsis"}
                  whiteSpace={"nowrap"}
                >
                  {config.path}
                </Text>
              </Td>
              <Td>{config.name}</Td>
              <Td>{config.used_count}</Td>
              <Td>
                <HStack spacing="6px">
                  <IconButton
                    onClick={() => {
                      dispatch(invokeNewTunnel(config.path));
                    }}
                    aria-label="Search database"
                    icon={<LinkIcon />}
                  />
                  <IconButton
                    onClick={() => {
                      dispatch(invokeRemoveConfiguration(config.path));
                    }}
                    aria-label="Search database"
                    icon={<DeleteIcon />}
                  />
                </HStack>
              </Td>
            </Tr>
          ))}
        </Tbody>
      </Table>
    </TableContainer>
  );
};

const renderSessionStatus = (status: LastSessionStatus) => {
  // Use chakra ui badges with proper colors
  switch (status.minor_code) {
    case 1:
      return <Badge colorScheme="red">CFG_ERROR</Badge>;
    case 2:
      return <Badge colorScheme="green">CFG_OK</Badge>;
    case 3:
      return <Badge colorScheme="orange">CFG_INLINE_MISSING</Badge>;
    case 4:
      return <Badge colorScheme="orange">CFG_REQUIRE_USER</Badge>;
    case 5:
      return <Badge colorScheme="blue">CONN_INIT</Badge>;
    case 6:
      return <Badge colorScheme="blue">CONN_CONNECTING</Badge>;
    case 7:
      return <Badge colorScheme="blue">CONN_CONNECTED</Badge>;
    case 8:
      return <Badge colorScheme="blue">CONN_DISCONNECTING</Badge>;
    case 9:
      return <Badge colorScheme="blue">CONN_DISCONNECTED</Badge>;
    case 10:
      return <Badge colorScheme="red">CONN_FAILED</Badge>;
    case 11:
      return <Badge colorScheme="red">CONN_AUTH_FAILED</Badge>;
    case 12:
      return <Badge colorScheme="blue">CONN_RECONNECTING</Badge>;
    case 13:
      return <Badge colorScheme="blue">CONN_PAUSING</Badge>;
    case 14:
      return <Badge colorScheme="blue">CONN_PAUSED</Badge>;
    case 15:
      return <Badge colorScheme="blue">CONN_RESUMING</Badge>;
    case 16:
      return <Badge colorScheme="green">CONN_DONE</Badge>;
    case 17:
      return <Badge colorScheme="purple">SESS_NEW</Badge>;
    case 18:
      return <Badge colorScheme="purple">SESS_BACKEND_COMPLETED</Badge>;
    case 19:
      return <Badge colorScheme="purple">SESS_REMOVED</Badge>;
    case 20:
      return <Badge colorScheme="purple">SESS_AUTH_USERPASS</Badge>;
    case 21:
      return <Badge colorScheme="purple">SESS_AUTH_CHALLENGE</Badge>;
    case 22:
      return <Badge colorScheme="purple">SESS_AUTH_URL</Badge>;
    case 23:
      return <Badge colorScheme="teal">PKCS11_SIGN</Badge>;
    case 24:
      return <Badge colorScheme="teal">PKCS11_ENCRYPT</Badge>;
    case 25:
      return <Badge colorScheme="teal">PKCS11_DECRYPT</Badge>;
    case 26:
      return <Badge colorScheme="teal">PKCS11_VERIFY</Badge>;
    case 27:
      return <Badge colorScheme="yellow">PROC_STARTED</Badge>;
    case 28:
      return <Badge colorScheme="yellow">PROC_STOPPED</Badge>;
    case 29:
      return <Badge colorScheme="yellow">PROC_KILLED</Badge>;
    default:
      return null;
  }
};

const SessionsList = () => {
  const sessions = useSelector(getAllSessions);
  const sessionsStatus = useSelector(getSessionsStatus);
  const dispatch = useDispatch();

  return (
    <TableContainer>
      <Table layout={"fixed"} size="sm" variant="simple">
        <TableCaption>Sessions</TableCaption>
        <Thead>
          <Tr>
            <Th w={350}>Path</Th>
            <Th>Status</Th>
            <Th w={100}>Actions</Th>
          </Tr>
        </Thead>
        <Tbody>
          {sessions.map((session) => (
            <Tr key={session.path}>
              <Td>
                <Text
                  overflow="hidden"
                  textOverflow={"ellipsis"}
                  whiteSpace={"nowrap"}
                >
                  {session.path}
                </Text>
              </Td>
              <Td>{renderSessionStatus(sessionsStatus[session.path])}</Td>
              <Td>
                <HStack spacing="6px">
                  <IconButton
                    onClick={() => {
                      dispatch(invokeConnectSession(session.path));
                    }}
                    aria-label="Search database"
                    icon={<LinkIcon />}
                  />
                  <IconButton
                    onClick={() => {
                      dispatch(invokeDisconnectSession(session.path));
                    }}
                    aria-label="Search database"
                    icon={<DeleteIcon />}
                  />
                </HStack>
              </Td>
            </Tr>
          ))}
        </Tbody>
      </Table>
    </TableContainer>
  );
};

const App = () => {
  return (
    <Flex justifyItems={"stretch"} direction={"column"} h={"100%"}>
      <Box flex={2} margin="12px">
        <Flex>
          <Box flex="1">
            <ImportConfigurationForm />
          </Box>
          <Box flex="2" marginLeft="6px" minW={0}>
            <Flex direction={"column"}>
              <ConfigurationsList />
              <SessionsList />
            </Flex>
          </Box>
        </Flex>
      </Box>
      <Box
        minH={0}
        marginLeft="12px"
        marginRight="12px"
        marginBottom="12px"
        flex={1}
      >
        <LogsContainer />
      </Box>
    </Flex>
  );
};

export default App;
