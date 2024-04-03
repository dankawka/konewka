import { useDispatch, useSelector } from "react-redux";
import {
  getCurrentModal,
  invokeConfirmExit,
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
} from "@chakra-ui/react";

export const Modals = () => {
  const currentModal = useSelector(getCurrentModal);
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

    return (
      <Modal isOpen={currentModal === "exit_confirmation"} onClose={closeModal}>
        <ModalOverlay />
        <ModalContent>
          <ModalHeader>Confirm quit</ModalHeader>
          <ModalCloseButton />
          <ModalBody>
            Closing the app will also disconnect all active sessions. Are you
            sure?
          </ModalBody>

          <ModalFooter>
            <Button colorScheme="blue" mr={3} onClick={closeModal}>
              Cancel
            </Button>
            <Button
              colorScheme="blue"
              variant="outline"
              onClick={handleConfirmExit}
            >
              Disconnect sessions and close the app
            </Button>
          </ModalFooter>
        </ModalContent>
      </Modal>
    );
  }
};
