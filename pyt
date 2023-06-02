// Copyright © 2016 Zhiming Wang <zmwangx@gmail.com>
//
// This work is free. You can redistribute it and/or modify it under the terms
// of the Do What The Fuck You Want To Public License, Version 2, as published
// by Sam Hocevar.
//
///////////////////////////////////////////////////////////////////////////////
//
// This program demonstrates how to correctly use openpty(3) from libutil on
// Linux to execute a command that must be run in a tty, and capture its
// (arbitrarily long) output.
//
// All error checking has been suppressed for conciseness.
//
// I figured out this solution while developing https://github.com/zmwangx/pm.

#include <iostream>
#include <pty.h>
#include <string>
#include <sys/wait.h>
#include <unistd.h>

using namespace std;

void openpty_demo(const char *output_size) {
  int master;
  int slave;
  openpty(&master, &slave, NULL, NULL, NULL);

  // Temporarily redirect stdout to the slave, so that the command executed in
  // the subprocess will write to the slave.
  int _stdout = dup(STDOUT_FILENO);
  dup2(slave, STDOUT_FILENO);

  pid_t pid = fork();
  if (pid == 0) {
    // We use
    //
    //     head -c $output_size /dev/zero
    //
    // as the command for our demo.
    const char *argv[] = {"head", "-c", output_size, "/dev/zero", NULL};
    execvp(argv[0], const_cast<char *const *>(argv));
  }

  fd_set rfds;
  struct timeval tv{0, 0};
  char buf[4097];
  ssize_t size;
  size_t count = 0;

  // Read from master as we wait for the child process to exit.
  //
  // We don't wait for it to exit and then read at once, because otherwise the
  // command being executed could potentially saturate the slave's buffer and
  // stall.
  while (1) {
    if (waitpid(pid, NULL, WNOHANG) == pid) {
      break;
    }
    FD_ZERO(&rfds);
    FD_SET(master, &rfds);
    if (select(master + 1, &rfds, NULL, NULL, &tv)) {
      size = read(master, buf, 4096);
      buf[size] = '\0';
      count += size;
    }
  }

  // Child process terminated; we flush the output and restore stdout.
  fsync(STDOUT_FILENO);
  dup2(_stdout, STDOUT_FILENO);

  // Read until there's nothing to read, by which time we must have read
  // everything because the child is long gone.
  while (1) {
    FD_ZERO(&rfds);
    FD_SET(master, &rfds);
    if (!select(master + 1, &rfds, NULL, NULL, &tv)) {
      // No more to read.
      break;
    }
    size = read(master, buf, 4096);
    buf[size] = '\0';
    count += size;
  }

  // Close both ends of the pty.
  close(master);
  close(slave);

  cout << "Total characters read: " << count << endl;
}

int main(int argc, const char *argv[]) {
  openpty_demo(argv[1]);
}

