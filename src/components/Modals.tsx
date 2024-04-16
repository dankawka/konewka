import { useDispatch, useSelector } from "react-redux";
import {
  getCurrentModal,
  getHasActiveSession,
  invokeConfirmExit,
  invokeMinimizeToTray,
  setCurrentModal,
} from "../store/features/common/common";
import {
  Modal,
  ModalOverlay,
  ModalContent,
  ModalHeader,
  ModalCloseButton,
  ModalBody,
  ModalFooter,
  Button,
  Flex,
} from "@chakra-ui/react";

export const Modals = () => {
  const currentModal = useSelector(getCurrentModal);
  const hasActiveSession = useSelector(getHasActiveSession);
  const dispatch = useDispatch();

  if (currentModal === null) {
    return null;
  }

  if (currentModal === "exit_confirmation") {
    const closeModal = () => {
      dispatch(setCurrentModal(null));
    };

    const handleConfirmExit = () => {
      dispatch(setCurrentModal(null));
      dispatch(invokeConfirmExit());
    };

    const handleMinimizeToTray = () => {
      dispatch(setCurrentModal(null));
      dispatch(invokeMinimizeToTray());
    };

    return (
      <Modal size="lg" isOpen={currentModal === "exit_confirmation"} onClose={closeModal}>
        <ModalOverlay />
        <ModalContent>
          <ModalHeader>Confirm quit</ModalHeader>
          <ModalCloseButton />
          <ModalBody>
            {hasActiveSession
              ? "Closing the app will also disconnect all active sessions. Are you sure?"
              : "Are you sure you want to close the app?"}
          </ModalBody>

          <ModalFooter>
            <Flex direction="column" w={"100%"}>
              <Flex>
                <Button
                  flex={3}
                  colorScheme="blue"
                  mr={3}
                  onClick={closeModal}
                >
                  Cancel
                </Button>
                <Button
                  flex={2}
                  colorScheme="blue"
                  variant="outline"
                  onClick={handleConfirmExit}
                >
                  {hasActiveSession ? "Disconnect and quit" : "Quit"}
                </Button>
              </Flex>
              <Button mt={3} colorScheme="green" onClick={handleMinimizeToTray}>
                Minimize to tray
              </Button>
            </Flex>
          </ModalFooter>
        </ModalContent>
      </Modal>
    );
  }
};
