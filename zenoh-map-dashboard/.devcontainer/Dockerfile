
# Install the .NET Core tool as before
FROM mcr.microsoft.com/dotnet/core/sdk as builder

RUN dotnet tool install --global GitVersion.Tool --version 5.*

ENV PATH="/root/.dotnet/tools:${PATH}"

FROM mcr.microsoft.com/devcontainers/typescript-node:1-20-bullseye
ENV DEBIAN_FRONTEND=noninteractive
RUN apt update && apt-get install -y iputils-ping
RUN wget https://dot.net/v1/dotnet-install.sh -O /tmp/dotnet-install.sh
RUN chmod +x /tmp/dotnet-install.sh
RUN /tmp/dotnet-install.sh --version latest
COPY --from=builder /root/.dotnet/tools/ /opt/bin

ENV PATH="/root/.dotnet/tools:${PATH}"
ENV DOTNET_ROOT="/root/.dotnet"
ENV PATH="/opt/bin:${PATH}:/root/.dotnet/tools:${DOTNET_ROOT}"
RUN npm install -g @vue/cli