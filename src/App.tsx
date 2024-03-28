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
  getAllConfigs,
  getAllSessions,
} from "./store/features/local-configs/local-configs";
import { ImportConfigurationPayload } from "./common/types";

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
      <Table variant="simple">
        <TableCaption>Imported configurations</TableCaption>
        <Thead>
          <Tr>
            <Th>Path</Th>
            <Th>Name</Th>
            <Th>Use count</Th>
            <Th>Actions</Th>
          </Tr>
        </Thead>
        <Tbody>
          {configs.map((config) => (
            <Tr key={config.path}>
              <Td>{config.path}</Td>
              <Td>{config.name}</Td>
              <Td>{config.used_count}</Td>
              <Td>
                <HStack spacing="6px">
                  <IconButton
                    onClick={() => {
                      dispatch(invokeRemoveConfiguration(config.path));
                    }}
                    aria-label="Search database"
                    icon={<DeleteIcon />}
                  />
                  <IconButton
                    onClick={() => {
                      dispatch(invokeNewTunnel(config.path));
                    }}
                    aria-label="Search database"
                    icon={<LinkIcon />}
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

const SessionsList = () => {
  const sessions = useSelector(getAllSessions);
  const dispatch = useDispatch();

  return (
    <TableContainer>
      <Table variant="simple">
        <TableCaption>Active sessions</TableCaption>
        <Thead>
          <Tr>
            <Th>Path</Th>
          </Tr>
        </Thead>
        <Tbody>
          {sessions.map((session) => (
            <Tr key={session.path}>
              <Td>{session.path}</Td>
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
    <Box marginLeft="12px" marginRight="12px" marginTop="12px">
      <Flex>
        <Box flex="1">
          <ImportConfigurationForm />
        </Box>
        <Box flex="2" marginLeft='6px'>
          <ConfigurationsList />
          <SessionsList />
        </Box>
      </Flex>
    </Box>
  );
};

export default App;
