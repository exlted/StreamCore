FROM node:16

WORKDIR /usr/src/app    
RUN apt update
RUN apt-get install -y fonts-liberation 
RUN apt-get install -y gconf-service 
RUN apt-get install -y libappindicator1 
RUN apt-get install -y libasound2 
RUN apt-get install -y libatk1.0-0 
RUN apt-get install -y libcairo2 
RUN apt-get install -y libcups2 
RUN apt-get install -y libfontconfig1 
RUN apt-get install -y libgbm-dev 
RUN apt-get install -y libgdk-pixbuf2.0-0 
RUN apt-get install -y libgtk-3-0 
RUN apt-get install -y libicu-dev 
RUN apt-get install -y libjpeg-dev 
RUN apt-get install -y libnspr4 
RUN apt-get install -y libnss3 
RUN apt-get install -y libpango-1.0-0 
RUN apt-get install -y libpangocairo-1.0-0 
RUN apt-get install -y libpng-dev 
RUN apt-get install -y libx11-6 
RUN apt-get install -y libx11-xcb1 
RUN apt-get install -y libxcb1 
RUN apt-get install -y libxcomposite1 
RUN apt-get install -y libxcursor1 
RUN apt-get install -y libxdamage1 
RUN apt-get install -y libxext6 
RUN apt-get install -y libxfixes3 
RUN apt-get install -y libxi6 
RUN apt-get install -y libxrandr2 
RUN apt-get install -y libxrender1 
RUN apt-get install -y libxss1 
RUN apt-get install -y libxtst6 
RUN apt-get install -y xdg-utils

COPY package*.json ./
RUN npm ci --only=production

COPY . .

CMD [ "node", "src/main.js"]